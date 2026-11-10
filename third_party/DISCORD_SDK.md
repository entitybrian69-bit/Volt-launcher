# Discord Social SDK — how official builds get it

The SDK is proprietary (per-application, non-redistributable), so it never
enters this public repository in usable form. Official builds bundle it like
this:

1. **Download** — Discord Developer Portal → the *Volt Launcher* application
   (ID `1569578048133533747`) → *Discord Social SDK* → *Getting Started*
   (submit the enable form once) → *Downloads* → version **1.10 or newer** →
   the **Android** package (`discord_partner_sdk.aar`). 1.10+ is required:
   it added Rich Presence over the unauthenticated transport on Android,
   which is the only mode Volt uses (no account linking, no tokens).
2. **Encrypt** — the AAR is encrypted to `third_party/discord_social_sdk.aar.enc`:

   ```sh
   openssl enc -aes-256-cbc -pbkdf2 -iter 200000 -pass env:DISCORD_SDK_KEY \
     -in discord_partner_sdk.aar -out third_party/discord_social_sdk.aar.enc
   ```

   Only this ciphertext is committed; without the key it is random bytes,
   so nothing redistributable is published.
3. **Key** — the same passphrase lives in the repository's Actions secret
   `DISCORD_SDK_KEY` (Settings → Secrets and variables → Actions).
4. **CI** — the *Add Discord Social SDK when configured* step of the Android
   APK workflow decrypts it into the gitignored `packaging/android/libs/`;
   Gradle bundles the AAR (native library + its Java classes) into the APK.
   Missing secret or missing ciphertext = the step no-ops and the APK ships
   without any Discord code.

Runtime behavior is documented in
`crates/horizon-client/src/discord_presence.rs`: HorizonActivity loads the
library when present, the presence thread drives the SDK's C interface
(`Discord_Client_SetApplicationId` + `Discord_Client_UpdateRichPresence`,
never `Authorize`/`Connect`), and the Discord app's absence makes everything
a quiet no-op.

Rich Presence art: upload a 1024×1024 logo named `volt` under the
application's *Rich Presence → Art Assets* in the Developer Portal.
