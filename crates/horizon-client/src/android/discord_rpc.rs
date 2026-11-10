//! Rich Presence through the signed-in Discord app on this phone: the Discord Android app
//! exports the bound service its Social SDK uses internally
//! (`com.discord.socialsdk.rpc.IDiscordRpcService`, reverse-engineered by Nightdavisao's
//! antisocialcord-kt, MIT — THIRD_PARTY_NOTICES.md), speaking the same JSON `SET_ACTIVITY`
//! frames as Discord's desktop IPC socket. Only the public application id is sent: no account
//! linking, no `Authorize`, no tokens, and no proprietary SDK binary in the APK.
//!
//! `VoltDiscordRpc.java` holds the Android plumbing (binding, main-thread callbacks); this
//! module is the [`Backend`] the presence thread drives: it wraps the activity JSON that
//! `discord_presence::activity` already builds into frames and pushes them over JNI. Some
//! Discord builds reject the optional `buttons`/`party` fields with an ERROR frame; the Java
//! side flags that and [`Rpc::pump`] resends the activity without them.
use std::io;
use std::sync::OnceLock;
use std::time::Duration;

use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JObject, JValue},
};
use serde_json::{Value, json};

use horizon_client::discord_presence::{self, Backend};

/// Mirrors the constants in `VoltDiscordRpc.java`.
const CONNECTED: i32 = 2;
const FAILED: i32 = 4;

struct Java {
    vm: JavaVM,
    /// `dev.horizon.android.VoltDiscordRpc`, resolved through the activity's class loader
    /// (plain `FindClass` on a native thread only sees system classes).
    class: GlobalRef,
    /// The application context `open` binds the service with.
    context: GlobalRef,
}
static JAVA: OnceLock<Java> = OnceLock::new();

/// Caches the JNI handles and registers this transport; called once from `android_main`,
/// before any game start. Quietly does nothing if the bridge class cannot be resolved.
pub fn initialize(vm: JavaVM, activity: &JObject) {
    let java = (|| -> jni::errors::Result<Java> {
        let mut env = vm.attach_current_thread()?;
        let java = env.with_local_frame_returning_local(16, |env| {
            let loader = env
                .call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
                .l()?;
            let name = JObject::from(env.new_string("dev.horizon.android.VoltDiscordRpc")?);
            env.call_method(
                &loader,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                &[JValue::Object(&name)],
            )?
            .l()
        })?;
        let class = env.new_global_ref(&java)?;
        let context = env
            .call_method(
                activity,
                "getApplicationContext",
                "()Landroid/content/Context;",
                &[],
            )?
            .l()?;
        let context = env.new_global_ref(&context)?;
        let vm = env.get_java_vm()?;
        Ok(Java { vm, class, context })
    })();
    match java {
        Ok(java) => {
            if JAVA.set(java).is_ok() {
                discord_presence::register_backend(connect);
            }
        }
        Err(error) => log::warn!("Volt: no Discord presence bridge: {error}"),
    }
}

/// A short launcher-facing status for the About tab, so "presence is not
/// showing" is diagnosable on the phone itself.
pub fn status_text() -> &'static str {
    if JAVA.get().is_none() {
        return "Unavailable on this device";
    }
    match bridge_state() {
        1 => "Connecting to the Discord app...",
        2 | 3 => "Connected through the Discord app",
        FAILED => "Not connected - Discord app missing or outdated",
        _ => "Waiting for the Discord app...",
    }
}

/// Runs `f` with the bridge class (as a `&GlobalRef` class descriptor) on an attached JNI env.
fn with_java<R>(f: impl FnOnce(&mut JNIEnv<'_>, &GlobalRef) -> jni::errors::Result<R>) -> Option<R> {
    let java = JAVA.get()?;
    let mut env = java.vm.attach_current_thread().ok()?;
    let result = env.with_local_frame(32, |env| {
        let out = f(env, &java.class);
        if out.is_err() {
            let _ = env.exception_clear();
        }
        out
    });
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            log::debug!("Volt: Discord presence bridge: {error}");
            None
        }
    }
}

fn bridge_state() -> i32 {
    with_java(|env, class| env.call_static_method(class, "state", "()I", &[])?.i()).unwrap_or(FAILED)
}

fn extended_rejected() -> bool {
    with_java(|env, class| {
        env.call_static_method(class, "extendedRejected", "()Z", &[])?
            .z()
    })
    .unwrap_or(false)
}

fn send_frame(frame: &str) -> bool {
    with_java(|env, class| {
        let frame = JObject::from(env.new_string(frame)?);
        env.call_static_method(
            class,
            "sendFrame",
            "(Ljava/lang/String;)Z",
            &[JValue::Object(&frame)],
        )?
        .z()
    })
    .unwrap_or(false)
}

fn bridge_close() {
    let _ = with_java(|env, class| {
        env.call_static_method(class, "close", "()V", &[])?.v()
    });
}

/// The [`BackendConnect`](discord_presence::BackendConnect) this module registers.
fn connect() -> Option<Box<dyn Backend>> {
    let java = JAVA.get()?;
    // The presence thread lives for the whole session: keep it attached so the once-a-second
    // pump never churns ART AttachCurrentThread/DetachCurrentThread.
    let _ = java.vm.attach_current_thread_permanently();
    let id: i64 = discord_presence::DISCORD_APP_ID.parse().ok()?;
    let state = with_java(|env, class| {
        env.call_static_method(
            class,
            "open",
            "(Landroid/content/Context;J)I",
            &[JValue::Object(java.context.as_obj()), JValue::Long(id)],
        )?
        .i()
    })?;
    if state == FAILED {
        return None; // No Discord app on this phone, or it refused the application id.
    }
    // Binding and connecting finish on the main thread within moments; frames sent before
    // READY wait on the Java side either way.
    // A cold Discord process can take several seconds to spawn and bind on a
    // low-end phone; waiting under ten seconds here beats failing and holding
    // the 15-second retry.
    let mut state = state;
    for _ in 0..100 {
        if state >= CONNECTED {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
        state = bridge_state();
    }
    if state < CONNECTED || state == FAILED {
        bridge_close();
        return None;
    }
    log::info!("Volt: Discord presence through the Discord app on this phone");
    Some(Box::new(Rpc {
        last: None,
        stripped: extended_rejected(),
        nonce: 0,
    }))
}

/// Lives and dies on the presence thread; never crosses threads.
struct Rpc {
    /// The full activity last published, for the stripped resend after an ERROR frame.
    last: Option<Value>,
    /// Whether activities are already sent without the optional fields.
    stripped: bool,
    nonce: u64,
}

impl Rpc {
    fn frame(&mut self, activity: Option<&Value>) -> String {
        self.nonce += 1;
        let mut args = json!({ "pid": 1 });
        if let Some(activity) = activity {
            args["activity"] = activity.clone();
        }
        json!({
            "cmd": "SET_ACTIVITY",
            "nonce": self.nonce.to_string(),
            "args": args,
        })
        .to_string()
    }

    fn push(&mut self, activity: Option<&Value>) -> io::Result<()> {
        let frame = self.frame(activity);
        if bridge_state() == FAILED || !send_frame(&frame) {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "the Discord app closed the RPC connection",
            ));
        }
        Ok(())
    }
}

/// `buttons` and `party` are optional per the service; some Discord builds reject them.
fn stripped(activity: &Value) -> Value {
    let mut activity = activity.clone();
    if let Some(map) = activity.as_object_mut() {
        map.remove("buttons");
        map.remove("party");
    }
    activity
}

impl Backend for Rpc {
    fn set_activity(&mut self, activity: Option<&Value>) -> io::Result<()> {
        self.last = activity.cloned();
        if !self.stripped {
            self.stripped = extended_rejected();
        }
        match (activity, self.stripped) {
            (Some(a), true) => self.push(Some(&stripped(a))),
            (other, _) => self.push(other),
        }
    }

    fn pump(&mut self) {
        if !self.stripped && extended_rejected() {
            // This Discord build just rejected the optional fields: publish again without
            // them so the card never stays blank until the next refresh.
            self.stripped = true;
            if let Some(last) = self.last.take() {
                let _ = self.push(Some(&stripped(&last)));
                self.last = Some(last);
            }
        }
    }

    fn close(self: Box<Self>) {
        // The clear frame the presence thread just sent crosses two processes; give it a
        // moment before the unbind tears the connection down.
        std::thread::sleep(Duration::from_millis(250));
        bridge_close();
    }
}
