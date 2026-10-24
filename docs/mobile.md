# Bridget for Android

[English](mobile.md) · [Français](mobile.fr.md)

Use Bridget on your phone to find existing agents, send instructions, read results,
answer questions and manage supported task actions. Select one machine or the
whole federation. The phone has its own revocable human authorization; it never
borrows an agent identity. Direct agent creation is not available from the phone.

## Connect your phone

1. Install the signed `bridget-android.apk` from the official Bridget release.
   Android asks you to allow installation from the browser once.
2. On the trusted master installation, run `bridget mobile pair --all` in an
   interactive terminal. The terminal displays the QR code. Use `--scope-file`
   with a private absolute file path for narrower permissions.
3. Open Bridget on Android, choose **Scan QR**, and scan that code. A code expires
   after five minutes and can associate only one device.
4. Choose a machine, a project and an explicit agent. Write your instruction.
   Stored, received, delivered and completed are different states in the app.

The master administrator can use `bridget mobile devices` and
`bridget mobile revoke DEVICE_UUID`. The Android settings also expose device
management. Forgetting a local authorization does not revoke the server record.

## Work after a network interruption

A command is saved before sending. If its response is lost, use its receipt
lookup first. A strict retry keeps the original key, time and content. The app
never invents another key to hide an unknown outcome. Expired duplicate protection
requires an explicit decision. Drafts, pending commands and association data use
Android Keystore encryption. Allowed attachments are limited to 8 MiB each and a
32 MiB local file budget; executable files and arbitrary disk paths are rejected.

A task action may be an instruction to its parent rather than a confirmed provider
action. Read the receipt and current task state. A recorded answer is distinct
from the agent acknowledging that answer. Missing history and disconnected
machines remain visible; cached observations are not live state.

## Notifications and quiet hours

Notifications are optional. Enable them in Android settings and accept the Android
permission prompt. Quiet hours follow the phone's local time. Messages stay
available during that period. A notification opens Bridget and the app reads the
current authorized state. A notification is never proof that a task completed.

Firebase Cloud Messaging receives only a version and an association binding hash,
not message text, file contents, agent names or provider secrets. Google still
operates the notification transport. The APK's public Firebase application
configuration must match the server account project. No Analytics SDK is included.
Missing configuration or permission is reported; interactive access still works.

## Server administrator

Use one permanent Bridget master to keep server agents available when a desktop
is offline. A reverse proxy alone does not move the federation authority. Preserve
identities, native receipts, task placements and origin attestations during any
migration. Never copy an open database or start a second master for the same
federation. Complete the migration checks before replacing the active master.

Configure the existing master from an interactive local terminal:

```sh
bridget mobile configure --public-url https://bridget.example.net
bridget mobile configure-notifications --service-account-file /home/operator/private/fcm-account.json
bridget mobile serve
```

The account file stays private in mode 0600. Use a dedicated account with only
`roles/firebasecloudmessaging.admin`. Billing is not needed for FCM. The trusted
proxy publishes only the loopback listener, normally `127.0.0.1:9467`. It must
replace the source-address header, not trust a header supplied by the client:

```caddyfile
bridget.example.net {
    request_body {
        max_size 12MB
    }
    reverse_proxy 127.0.0.1:9467 {
        header_up X-Bridget-Client-IP {remote_host}
    }
}
```

Run `bridget mobile serve` with the existing native service manager. Keep its
namespace and socket identical to the master. Authentication precedes private
reads and effects. Pairing uses a per-source budget; renewal uses a separate
budget for a known device. Connections, workers, uploads and subscriptions are
bounded. The proxy header is not an authentication credential.

For a workstation connecting to a server master, the existing federation installer
accepts `--direction to-master`. `--root` and `--socket` then identify the local
client namespace; `--remote-root` and `--remote-socket` identify the remote master.
It creates a private Unix listener and uses authenticated SSH standard streams.
There is no public native TCP port. `socat` is required on both machines. An
existing link keeps its stored direction unless an explicit matching direction
is supplied; the CLI displays the direction it reuses. See the
[SSH federation guide](federation-services.md) for installation parameters.

## Updates and recovery

Settings checks the official GitHub release metadata. Only the stable
`bridget-android.apk` asset in the Bridget repository is offered. Checking for an
update sends no device credential. An unavailable release is reported separately
from an up-to-date app. The browser downloads the APK; Android verifies its
signature and asks before installation. Keep application data when updating.

Back up the release signing key outside Git. Never uninstall the app merely to
solve a signature mismatch: that removes local data. A developer installation
requires a verified signing-key transition before its first production update.
The Android version follows the workspace version. Build with the private
`BRIDGET_FIREBASE_CONFIG` file when notifications are enabled; it contains public
application options, never the server account key. Retain the current installed
version and one verified rollback package.
