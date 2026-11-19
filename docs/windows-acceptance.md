# Windows 11: supervised, licensed acceptance

This is a **human-assisted** Actions job. It does not supply a Microsoft account,
create a Windows VM, reuse a player profile, or publish a package. A green hosted
Windows navigation test is not this test.

## Administrator checklist

Use either the local route below or the Actions route, **not both for the same run**.
The local route is preferable if the public repository cannot safely restrict runner access.

1. Use a **disposable Windows 11 x64 client VM** with an interactive GPU desktop,
   adequate disk for the Rust build and Minecraft installation, and a fresh,
   non-admin local user. Install x64 PowerShell 7, Git, Python 3.11+, Visual
   Studio C++ Build Tools (MSVC x64 and Windows SDK), and rustup with the
   repository's `rust-toolchain.toml` and `x86_64-pc-windows-msvc` target.
   Allow the official Minecraft and Microsoft HTTPS endpoints. Keep the console
   signed in and unlocked. Session zero, Windows Server and ARM64 do not qualify.
2. **Public fork precautions:** never attach a personal machine, persistent runner,
   or one with access to private network resources. Restrict execution to trusted,
   reviewed workflow code; a label alone is *not* an access control. Require
   approval of outside contributors' workflows. Inspect the queued job and exact
   SHA before registration and again before using Microsoft sign-in. If you cannot
   isolate the VM and prevent untrusted jobs, do not register it. Use a separate
   browser/device for the Microsoft verification page and your own legitimately
   licensed test account. Never send a device code, token, password, or 2FA in chat.
3. In the Microsoft Entra app registration, confirm it supports **personal Microsoft
   accounts** and **Allow public client flows** is enabled under Authentication.
   The registration must already be approved for the Minecraft services used here;
   an Entra GUID by itself is not proof of that approval. Use its **Application
   (client) ID**, not the tenant ID, object ID, or a secret. Do not borrow another
   launcher's registration. No confidential-client secret is used by this app.
   Set the fork's Actions **variable** `VOLT_MSA_CLIENT_ID` to the approved public
   device-flow application ID (not a client secret). Dispatch the workflow on
   the reviewed branch, or use the licensed job already queued for that commit:

   ```sh
   gh workflow run windows-engine.yml --ref arena/d51c35d7-volt-launcherpc -f windows11=true -f licensed_play=true
   ```

4. A fork administrator opens **Settings → Actions → Runners → New self-hosted
   runner → Windows / x64**. Use the official runner ZIP and the independent
   SHA256 shown by GitHub for that version. From the *reviewed checkout* on the
   VM run the following; enter the short-lived repository registration token
   only at the local runner prompt. Leave **run as a service** off.

   ```powershell
   ./packaging/windows/start-test-runner.ps1 -RunnerZip $downloadedZip -ExpectedSHA256 $githubChecksum
   ```

   The script verifies the OS, session and archive, configures the fork URL and
   `volt-win11-x64-interactive` label, and runs **one** job in the foreground.
   Don't provide the runner with admin GitHub credentials. If the Actions queue
   expires, dispatch again rather than treating the missing result as a pass.
5. At **Human-assisted fresh licensed Play acceptance**, open Account in the GUI,
   sign in on Microsoft's page, return to Volt, and click Download/Play in the
   default profile. Keep the launcher open. Wait for verified installation and
   the game window. In the game, create a singleplayer world, move around for
   at least 30 seconds, save and quit the game normally. Do not start the engine
   manually. No game files or accounts should be copied in to help the test.
6. The job requires a **new empty home**, live ownership, a signed session, the
   actual sibling engine as a direct child of Volt, official install markers
   for the pinned version, at least 120 presented frames with the player in the
   world, a nonempty `level.dat`, and clean shutdown. The game's normal title,
   world creation and render paths run unchanged. Tokens stay in process memory
   for this test; normal Credential Manager persistence needs a separate test.
   Logs, session data, world files and the raw smoke report are **not artifacts**.
   Destroy the VM even if the job fails (browser state and private logs may remain).

Only after successful licensed acceptance **and** the remaining feature and
regression gates, follow [private candidate verification](windows-port.md#candidate-packaging-and-windows-11-verification).
The package verifier now checks presented navigation from *both* deployed copies
as well as contents, install/uninstall and hashes. Nothing here uploads release assets.

## Local route (no GitHub runner or admin credentials on the Windows desktop)

1. Prepare the same clean VM, approved ID and licensed tester as above. On a
   **separate trusted workstation**, pick the successful `tables` job from the
   reviewed Windows workflow. Confirm `headSha` and the fork, then retrieve only
   its **generated input**, not an engine executable or a player profile:

   ```sh
   gh run view "$run" -R luciferstudios/Volt-launcherPc --json headSha,jobs,url
   gh run download "$run" -R luciferstudios/Volt-launcherPc -n _internal-volt-engine-tables -D tables
   ```

   Transfer the extracted `engine-data.zip` outside the clean checkout on the VM.
   If the temporary artifact expired, rerun the hosted workflow for the same source.
2. From x64 PowerShell 7 in the signed-in desktop, clone the public repository
   and check out the **exact reviewed SHA**. Use a new PowerShell process so the
   build environment is confined to the disposable session. Start the test:

   ```powershell
   ./packaging/windows/test-local-desktop.ps1 -ReviewedCommit $reviewedSha -PublicClientId $approvedAppId -EngineTables $tablesZip
   ```

   This builds the locked launcher and engine locally and calls the **same**
   attended Play verifier as Actions. Follow steps 5–6 above. Record commit,
   Windows build, pass/fail and the final non-private assertion summary. Do not
   upload the VM, profile or logs. Destroy the VM, including the scratch build.
   A local pass is desktop evidence, **not** a green Actions runner or a package test.

**If a job stays queued:** in Settings → Actions → Runners, verify an *online,
idle* runner has the exact custom label, its console is unlocked, and its foreground
`run.cmd` is still active. An ephemeral runner unregisters after one job; create
a new one for the next job. If the hosted preflight rejects the app ID, correct
the repository variable and dispatch a **new** run. If runner access is prohibited,
use the local route rather than weakening repository Actions permissions.
