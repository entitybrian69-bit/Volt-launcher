package dev.volt.launcher;

import android.os.Build;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;

/** SAF callback bridge. The native activity, UI and file operations stay Rust.
 *  Deliberately contains no PackageInstaller / REQUEST_INSTALL_PACKAGES code:
 *  update APKs download in the user's browser, so Play Protect never sees an
 *  "installs other apps" signal when players sideload Volt Launcher. */
public final class VoltActivity extends android.app.NativeActivity {
    static { System.loadLibrary("volt_launcher"); }

    private native void nativeDocumentResult(int request, int result, String uri);

    @Override protected void onCreate(android.os.Bundle state) {
        super.onCreate(state);
        keepBehindSystemBars();
    }

    /** From Android 15 (targetSdk 35) edge-to-edge is enforced: the status bar
     *  is drawn on top of the game's surface instead of above it, so the
     *  title bar and corner buttons of every screen collide with the clock,
     *  the wifi and the battery. Immersive mode hides those bars outright,
     *  which both brings back the pre-15 layout and stops the overlap.
     *  `setDecorFitsSystemWindows` is deliberately not touched: the window
     *  keeps `adjustResize`, so the soft keyboard still resizes the view. */
    private void keepBehindSystemBars() {
        android.view.Window window = getWindow();
        if (window == null) return;
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            WindowManager.LayoutParams attrs = window.getAttributes();
            attrs.layoutInDisplayCutoutMode =
                    WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES;
            window.setAttributes(attrs);
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            WindowInsetsController controller = window.getInsetsController();
            if (controller != null) {
                controller.hide(WindowInsets.Type.statusBars() | WindowInsets.Type.navigationBars());
                controller.setSystemBarsBehavior(
                        WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
            }
        } else {
            window.getDecorView().setSystemUiVisibility(
                    View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                            | View.SYSTEM_UI_FLAG_FULLSCREEN
                            | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                            | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                            | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                            | View.SYSTEM_UI_FLAG_LAYOUT_STABLE);
        }
    }

    /** A focus change (the IME, a permission dialog, returning from recents)
     *  clears a transient immersive state. Put the bars back away. */
    @Override public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) keepBehindSystemBars();
    }

    @Override protected void onActivityResult(int request, int result, android.content.Intent data) {
        super.onActivityResult(request, result, data);
        if (request != 4101) return;
        nativeDocumentResult(request, result,
                data == null || data.getData() == null ? null : data.getData().toString());
    }

    @Override protected void onNewIntent(android.content.Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
    }

    public static String supportedAbis() {
        StringBuilder sb = new StringBuilder();
        String[] abis = android.os.Build.SUPPORTED_ABIS;
        if (abis != null) {
            for (int i = 0; i < abis.length; i++) {
                if (abis[i] == null || abis[i].isEmpty()) continue;
                if (sb.length() > 0) sb.append(',');
                sb.append(abis[i]);
            }
        }
        return sb.toString();
    }

    public String appVersionName() {
        try {
            android.content.pm.PackageInfo info =
                    getPackageManager().getPackageInfo(getPackageName(), 0);
            return info != null && info.versionName != null ? info.versionName : "0.8.2";
        } catch (Throwable ignored) {
            return "0.8.2";
        }
    }
}
