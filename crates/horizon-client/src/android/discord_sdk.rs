//! Rich Presence through the signed-in Discord app on this phone, with the Social SDK's
//! unauthenticated transport (SDK 1.10+): only the public application id is set and activities
//! published — no account linking, no `Authorize`, no `Connect`, no tokens. The proprietary
//! `libdiscord_partner_sdk.so` is in the APK only when the owner's SDK package was present at
//! build time (HorizonActivity loads it so its JNI hooks see the app class loader); without
//! it, or without the Discord app installed, everything here is a quiet no-op and the presence
//! thread falls back to scanning for an IPC socket that Android never has.
//!
//! Signatures match `cdiscord.h` of the Social SDK: every object is one opaque pointer that
//! `*_Init` fills and `*_Drop` frees; `Discord_String` is a pointer + length (not NUL-ended);
//! strings passed in are copied by the SDK during the call. These declarations are original,
//! written against the SDK's published C interface.
use std::ffi::{CStr, c_char, c_int, c_void};
use std::io;
use std::sync::OnceLock;

use serde_json::Value;

use horizon_client::discord_presence::{self, Backend};

/// Hands the presence thread this transport; called once from `android_main`.
pub fn register() {
    discord_presence::register_backend(|| {
        Sdk::connect().map(|sdk| Box::new(sdk) as Box<dyn Backend>)
    });
}

#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(name: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

/// `Discord_String`: UTF-8 bytes + length. The SDK copies the bytes during each call.
#[repr(C)]
struct DStr {
    ptr: *mut u8,
    size: usize,
}
/// Every SDK object (`Discord_Client`, `Discord_Activity`, ...) is one opaque pointer.
#[repr(C)]
struct Handle {
    opaque: *mut c_void,
}

type InitFn = unsafe extern "C" fn(*mut Handle);
type UpdateCb = unsafe extern "C" fn(*mut c_void, *mut c_void);
type FreeFn = unsafe extern "C" fn(*mut c_void);

struct Api {
    run_callbacks: unsafe extern "C" fn(),
    client_init: InitFn,
    client_drop: InitFn,
    client_set_application_id: unsafe extern "C" fn(*mut Handle, u64),
    client_update_rich_presence: unsafe extern "C" fn(
        *mut Handle,
        *mut Handle,
        Option<UpdateCb>,
        Option<FreeFn>,
        *mut c_void,
    ),
    client_clear_rich_presence: unsafe extern "C" fn(*mut Handle),
    activity_init: InitFn,
    activity_drop: InitFn,
    activity_set_type: unsafe extern "C" fn(*mut Handle, i32),
    activity_set_details: unsafe extern "C" fn(*mut Handle, *mut DStr),
    activity_set_state: unsafe extern "C" fn(*mut Handle, *mut DStr),
    activity_set_timestamps: unsafe extern "C" fn(*mut Handle, *mut Handle),
    activity_add_button: unsafe extern "C" fn(*mut Handle, *const Handle),
    timestamps_init: InitFn,
    timestamps_drop: InitFn,
    timestamps_set_start: unsafe extern "C" fn(*mut Handle, u64),
    button_init: InitFn,
    button_drop: InitFn,
    button_set_label: unsafe extern "C" fn(*mut Handle, DStr),
    button_set_url: unsafe extern "C" fn(*mut Handle, DStr),
}

fn api() -> Option<&'static Api> {
    static API: OnceLock<Option<Api>> = OnceLock::new();
    API.get_or_init(load).as_ref()
}

fn load() -> Option<Api> {
    unsafe {
        // RTLD_NOW; already loaded by HorizonActivity, so this only takes a handle.
        let lib = dlopen(c"libdiscord_partner_sdk.so".as_ptr(), 2);
        if lib.is_null() {
            log::info!("Volt: this build carries no Discord Social SDK");
            return None;
        }
        let sym = |name: &CStr| -> Option<*mut c_void> {
            let p = dlsym(lib, name.as_ptr());
            if p.is_null() {
                log::warn!("Volt: Discord SDK misses {}", name.to_string_lossy());
            }
            (!p.is_null()).then_some(p)
        };
        macro_rules! f {
            ($name:literal) => {
                std::mem::transmute(sym($name)?)
            };
        }
        Some(Api {
            run_callbacks: f!(c"Discord_RunCallbacks"),
            client_init: f!(c"Discord_Client_Init"),
            client_drop: f!(c"Discord_Client_Drop"),
            client_set_application_id: f!(c"Discord_Client_SetApplicationId"),
            client_update_rich_presence: f!(c"Discord_Client_UpdateRichPresence"),
            client_clear_rich_presence: f!(c"Discord_Client_ClearRichPresence"),
            activity_init: f!(c"Discord_Activity_Init"),
            activity_drop: f!(c"Discord_Activity_Drop"),
            activity_set_type: f!(c"Discord_Activity_SetType"),
            activity_set_details: f!(c"Discord_Activity_SetDetails"),
            activity_set_state: f!(c"Discord_Activity_SetState"),
            activity_set_timestamps: f!(c"Discord_Activity_SetTimestamps"),
            activity_add_button: f!(c"Discord_Activity_AddButton"),
            timestamps_init: f!(c"Discord_ActivityTimestamps_Init"),
            timestamps_drop: f!(c"Discord_ActivityTimestamps_Drop"),
            timestamps_set_start: f!(c"Discord_ActivityTimestamps_SetStart"),
            button_init: f!(c"Discord_ActivityButton_Init"),
            button_drop: f!(c"Discord_ActivityButton_Drop"),
            button_set_label: f!(c"Discord_ActivityButton_SetLabel"),
            button_set_url: f!(c"Discord_ActivityButton_SetUrl"),
        })
    }
}

fn dstr(s: &str) -> DStr {
    DStr {
        ptr: s.as_ptr().cast_mut(),
        size: s.len(),
    }
}

/// `Discord_Client_UpdateRichPresenceCallback`: the result is the SDK's, nothing to free.
unsafe extern "C" fn updated(_result: *mut c_void, _user_data: *mut c_void) {}

/// Lives and dies on the presence thread; never crosses threads.
struct Sdk {
    client: Handle,
}

impl Sdk {
    /// Some only when the SDK library is in this APK and the application id parses.
    fn connect() -> Option<Self> {
        let api = api()?;
        let id: u64 = discord_presence::DISCORD_APP_ID.parse().ok()?;
        let mut client = Handle {
            opaque: std::ptr::null_mut(),
        };
        unsafe {
            (api.client_init)(&mut client);
            (api.client_set_application_id)(&mut client, id);
        }
        log::info!("Volt: Discord presence through the Social SDK on this phone");
        Some(Sdk { client })
    }
}

impl Backend for Sdk {
    /// Publishes the same JSON activity the IPC path sends; `None` clears it. Failures (no
    /// Discord app, not signed in) surface in the completion callback, which is ignored: the
    /// SDK keeps retrying its transport on its own.
    fn set_activity(&mut self, activity: Option<&Value>) -> io::Result<()> {
        let api = api().expect("connected only with the API loaded");
        let Some(a) = activity else {
            unsafe {
                (api.client_clear_rich_presence)(&mut self.client);
                (api.run_callbacks)();
            }
            return Ok(());
        };
        let details = a["details"].as_str().unwrap_or_default().to_owned();
        let state = a["state"].as_str().map(str::to_owned);
        unsafe {
            let mut act = Handle {
                opaque: std::ptr::null_mut(),
            };
            (api.activity_init)(&mut act);
            (api.activity_set_type)(&mut act, 0); // ActivityTypes::Playing
            let mut d = dstr(&details);
            (api.activity_set_details)(&mut act, &mut d);
            if let Some(state) = &state {
                let mut s = dstr(state);
                (api.activity_set_state)(&mut act, &mut s);
            }
            if let Some(start) = a["timestamps"]["start"].as_u64() {
                let mut ts = Handle {
                    opaque: std::ptr::null_mut(),
                };
                (api.timestamps_init)(&mut ts);
                // The IPC payload counts unix seconds; the SDK wants milliseconds.
                (api.timestamps_set_start)(&mut ts, start.saturating_mul(1000));
                (api.activity_set_timestamps)(&mut act, &mut ts);
                (api.timestamps_drop)(&mut ts);
            }
            for b in a["buttons"].as_array().into_iter().flatten() {
                let (Some(label), Some(url)) = (b["label"].as_str(), b["url"].as_str()) else {
                    continue;
                };
                let mut button = Handle {
                    opaque: std::ptr::null_mut(),
                };
                (api.button_init)(&mut button);
                (api.button_set_label)(&mut button, dstr(label));
                (api.button_set_url)(&mut button, dstr(url));
                (api.activity_add_button)(&mut act, &button);
                (api.button_drop)(&mut button);
            }
            (api.client_update_rich_presence)(
                &mut self.client,
                &mut act,
                Some(updated),
                None,
                std::ptr::null_mut(),
            );
            (api.activity_drop)(&mut act);
            (api.run_callbacks)();
        }
        Ok(())
    }

    /// Dispatches queued SDK completion callbacks; called about once a second while parked.
    fn pump(&mut self) {
        if let Some(api) = api() {
            unsafe { (api.run_callbacks)() }
        }
    }

    fn close(mut self: Box<Self>) {
        if let Some(api) = api() {
            unsafe {
                (api.client_clear_rich_presence)(&mut self.client);
                (api.run_callbacks)();
                (api.client_drop)(&mut self.client);
            }
        }
    }
}
