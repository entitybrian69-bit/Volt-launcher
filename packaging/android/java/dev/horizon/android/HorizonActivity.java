package dev.horizon.android;

/** SAF callback and PackageInstaller bridge. The native activity, UI and file operations stay Rust. */
public final class HorizonActivity extends android.app.NativeActivity {
    private static final String ACTION_PACKAGE_INSTALLED = "ae.net.Voltlauncher.PACKAGE_INSTALLED";
    static { System.loadLibrary("horizon_android"); }
    // Discord's Social SDK, when packaged: loading from Java runs its JNI hooks with the app
    // class loader; Rust then drives it for Rich Presence. A build without it is fine.
    static { try { System.loadLibrary("discord_partner_sdk"); } catch (Throwable ignored) {} }

    private String pendingUpdateApkPath;

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
        handleInstallerIntent(intent);
    }

    @Override protected void onResume() {
        super.onResume();
        handleInstallerIntent(getIntent());
        String pending = this.pendingUpdateApkPath;
        if (pending != null && (android.os.Build.VERSION.SDK_INT < 26
                || getPackageManager().canRequestPackageInstalls())) {
            this.pendingUpdateApkPath = null;
            installUpdateApk(pending);
        }
    }

    private void handleInstallerIntent(android.content.Intent intent) {
        if (intent == null || !ACTION_PACKAGE_INSTALLED.equals(intent.getAction())) return;
        intent.setAction(null);
        int status = intent.getIntExtra(android.content.pm.PackageInstaller.EXTRA_STATUS, -999);
        if (status == android.content.pm.PackageInstaller.STATUS_PENDING_USER_ACTION) {
            @SuppressWarnings("deprecation")
            android.content.Intent confirm =
                    intent.getParcelableExtra(android.content.Intent.EXTRA_INTENT);
            if (confirm != null) {
                confirm.addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK);
                startActivity(confirm);
            }
        }
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

    public String installUpdateApk(String apkPath) {
        try {
            java.io.File apkFile = new java.io.File(apkPath);
            if (!apkFile.isFile() || apkFile.length() <= 0) {
                return "ERR:Downloaded update APK was not found.";
            }
            if (android.os.Build.VERSION.SDK_INT >= 26
                    && !getPackageManager().canRequestPackageInstalls()) {
                this.pendingUpdateApkPath = apkPath;
                android.content.Intent perm = new android.content.Intent(
                        android.provider.Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,
                        android.net.Uri.parse("package:" + getPackageName()));
                perm.addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK);
                startActivity(perm);
                return "ALLOW_SOURCE";
            }
            this.pendingUpdateApkPath = null;
            android.content.pm.PackageInstaller installer =
                    getPackageManager().getPackageInstaller();
            android.content.pm.PackageInstaller.SessionParams params =
                    new android.content.pm.PackageInstaller.SessionParams(
                            android.content.pm.PackageInstaller.SessionParams.MODE_FULL_INSTALL);
            params.setAppPackageName(getPackageName());
            params.setSize(apkFile.length());
            int sessionId = installer.createSession(params);
            android.content.pm.PackageInstaller.Session session = installer.openSession(sessionId);
            try (java.io.InputStream in = new java.io.FileInputStream(apkFile);
                 java.io.OutputStream out =
                         session.openWrite("volt-update.apk", 0, apkFile.length())) {
                byte[] buf = new byte[65536];
                int n;
                while ((n = in.read(buf)) != -1) {
                    out.write(buf, 0, n);
                }
                session.fsync(out);
            } catch (Throwable t) {
                session.abandon();
                throw t;
            }
            android.content.Intent callback =
                    new android.content.Intent(this, HorizonActivity.class);
            callback.setAction(ACTION_PACKAGE_INSTALLED);
            callback.addFlags(android.content.Intent.FLAG_ACTIVITY_SINGLE_TOP
                    | android.content.Intent.FLAG_ACTIVITY_CLEAR_TOP);
            int flags = android.app.PendingIntent.FLAG_UPDATE_CURRENT;
            if (android.os.Build.VERSION.SDK_INT >= 31) {
                flags |= 0x02000000; // PendingIntent.FLAG_MUTABLE
            }
            android.app.PendingIntent pi =
                    android.app.PendingIntent.getActivity(this, sessionId, callback, flags);
            session.commit(pi.getIntentSender());
            session.close();
            return "OK";
        } catch (Throwable t) {
            return "ERR:" + t.getMessage();
        }
    }
}
