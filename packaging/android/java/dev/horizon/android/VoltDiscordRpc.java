package dev.horizon.android;

import android.content.ComponentName;
import android.content.Context;
import android.content.Intent;
import android.content.ServiceConnection;
import android.os.IBinder;

import com.discord.socialsdk.rpc.IDiscordRpcCallback;
import com.discord.socialsdk.rpc.IDiscordRpcConnection;
import com.discord.socialsdk.rpc.IDiscordRpcService;

/**
 * Binds the Discord app's Rich Presence service (the one its Social SDK uses internally;
 * interface reverse-engineered by Nightdavisao's antisocialcord-kt, MIT — THIRD_PARTY_NOTICES.md)
 * and relays the JSON frames Rust builds. Only the public application id crosses this bridge:
 * no account linking, no tokens. State machine and payloads stay in Rust
 * (crates/horizon-client/src/android/discord_rpc.rs); this class only holds the Android
 * plumbing a bound service needs: the connection callbacks arrive on the main thread, the
 * Rust presence thread polls {@link #state()} and pushes frames.
 */
public final class VoltDiscordRpc {
    private VoltDiscordRpc() {}

    public static final int IDLE = 0;
    public static final int BINDING = 1;
    public static final int CONNECTED = 2;
    public static final int READY = 3;
    public static final int FAILED = 4;

    private static final String ACTION = "com.discord.socialsdk.rpc.IDiscordRpcService";
    /** The official app first, then the main client mod that keeps the same service. */
    private static final String[] PACKAGES = {"com.discord", "cocobo1.pupu.app"};

    private static final Object LOCK = new Object();
    private static Context app;
    private static IDiscordRpcService service;
    private static IDiscordRpcConnection connection;
    private static int state = IDLE;
    private static boolean bound;
    private static boolean extendedRejected;
    private static long appId;
    private static String pending;

    private static final ServiceConnection binding = new ServiceConnection() {
        @Override public void onServiceConnected(ComponentName name, IBinder binder) {
            synchronized (LOCK) {
                service = IDiscordRpcService.Stub.asInterface(binder);
                connectLocked();
            }
        }
        @Override public void onServiceDisconnected(ComponentName name) {
            synchronized (LOCK) {
                service = null;
                connection = null;
                if (state != FAILED) state = IDLE;
            }
        }
    };

    private static final IDiscordRpcCallback callback = new IDiscordRpcCallback.Stub() {
        @Override public void onFrame(String frame) {
            if (frame == null) return;
            synchronized (LOCK) {
                if (frame.contains("\"READY\"")) {
                    if (state == CONNECTED) state = READY;
                    if (pending != null && connection != null) {
                        try { connection.sendFrame(pending); } catch (Throwable ignored) {}
                        pending = null;
                    }
                } else if (frame.contains("\"ERROR\"")) {
                    // Some Discord builds reject optional fields (buttons, party); Rust
                    // notices this flag and resends the activity without them.
                    extendedRejected = true;
                }
            }
        }
        @Override public void onClose(int code, String message) {
            synchronized (LOCK) {
                connection = null;
                pending = null;
                state = FAILED;
            }
        }
    };

    /** Starts binding and connecting for {@code id}; idempotent. Returns the current state. */
    public static int open(Context context, long id) {
        synchronized (LOCK) {
            app = context.getApplicationContext();
            appId = id;
            if (connection != null) return state;
            if (bound) {
                connectLocked();
                return state;
            }
            Intent intent = null;
            for (String pkg : PACKAGES) {
                Intent probe = new Intent(ACTION).setPackage(pkg);
                if (app.getPackageManager().resolveService(probe, 0) != null) {
                    intent = probe;
                    break;
                }
            }
            if (intent == null) {
                state = FAILED; // No Discord app on this phone.
                return state;
            }
            state = BINDING;
            try {
                bound = app.bindService(intent, binding, Context.BIND_AUTO_CREATE);
            } catch (Throwable t) {
                bound = false;
            }
            if (!bound) state = FAILED;
            return state;
        }
    }

    private static void connectLocked() {
        if (service == null || connection != null) return;
        try {
            connection = service.connect(appId, "1", callback);
            state = connection != null ? CONNECTED : FAILED;
        } catch (Throwable t) {
            connection = null;
            state = FAILED;
        }
    }

    public static int state() {
        synchronized (LOCK) { return state; }
    }

    public static boolean extendedRejected() {
        synchronized (LOCK) { return extendedRejected; }
    }

    /** Sends when connected; before READY the newest frame waits and is flushed on READY. */
    public static boolean sendFrame(String frame) {
        synchronized (LOCK) {
            if (connection == null) return false;
            if (state != READY) {
                pending = frame;
                return true;
            }
            try {
                connection.sendFrame(frame);
                return true;
            } catch (Throwable t) {
                return false;
            }
        }
    }

    /** Disconnects and unbinds; the next {@link #open} starts fresh. */
    public static void close() {
        synchronized (LOCK) {
            if (connection != null) {
                try { connection.disconnect(); } catch (Throwable ignored) {}
                connection = null;
            }
            if (bound && app != null) {
                try { app.unbindService(binding); } catch (Throwable ignored) {}
            }
            bound = false;
            service = null;
            pending = null;
            state = IDLE;
        }
    }
}
