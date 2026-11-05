# Bundled Horizon Zoom on Android

Volt includes the **official Horizon Zoom 2.0.0** Rust runtime in its Android
APK. This is a deliberately limited first native-mod installer, not a generic
`.hnmod` loader or a registry downloader: the reviewed code is compiled into
the APK before it is signed. There is no network download when you tap Install.
The Horizon Zoom project is distinct from Volt's Zoomify feature.

In the launcher, select a profile, open **Mods**, and find **Horizon Zoom
2.0.0** in **Horizon browser** (search “Zoom” if necessary). Tap **Install**
to verify and download the matching registry bundle, or choose **Volt mods**
to activate the bundled version without a download. This activates
only the selected profile; **Remove** deactivates it while retaining its
settings. In-game Options > Mods lists Horizon Zoom and supplies its on/off
switch, key binding, starting zoom, smoothing, and sensitivity controls.
The launcher reflects the in-game on/off state when the game returns. Installing
again turns it on. No game restart is needed after installation because the
action is available from the launcher, before starting the game.

Hold **Z** while playing. To use touch, add the **Horizon Zoom (Z)** button
in Controls Studio or use a preset that contains it; if the in-game key binding
is changed, choose the matching key in the touch layout. A mouse wheel adjusts
zoom while held. If both Zoomify and Horizon Zoom are active, Volt's Zoomify
has priority while its key is held.

The activation marker is profile-local. Removing it does not remove the code
from the APK or affect arbitrary user `.hnmod` files. Mod source and license
are pinned in `THIRD_PARTY_NOTICES.md`; no registry-sourced code is executed in
the signing pipeline. Other native mods still require separate integration,
license/security review, both-ABI builds and Android runtime testing. An APK
update must retain the Volt signing certificate and a valid versionCode.

Neither a successful APK build nor this documentation proves behavior on a
physical Android device. See `compatibility.md` for the isolated source probe
and its limitations.
