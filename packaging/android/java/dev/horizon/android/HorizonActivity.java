package dev.horizon.android;

/** SAF callback bridge. The native activity, UI and file operations stay Rust.
 *  Deliberately contains no PackageInstaller / REQUEST_INSTALL_PACKAGES code:
 *  update APKs download in the user's browser, so Play Protect never sees an
 *  "installs other apps" signal when players sideload Volt Launcher. */
public final class HorizonActivity extends android.app.NativeActivity {
    static { System.loadLibrary("horizon_android"); }

    private native void nativeDocumentResult(int request, int result, String uri);

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
