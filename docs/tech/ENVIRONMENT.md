---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: draft
---

# Environment

## Development targets

The required desktop targets are Windows 11 and macOS.
The current workspace is D:\Moo.exe\CowWorker. The user works with PowerShell 7 on Windows.
The local implementation uses Node.js 22.14.0, npm 11.18.0, Rust 1.97.1, and Cargo 1.97.1.
Windows development and release builds pass. The release build produces an NSIS installer.
The [README](../../README.md) contains startup and build commands.

## Local data

The desktop app stores workspaces under the Tauri local application directory.
Windows uses `%LOCALAPPDATA%/com.cowworker.desktop/dev` for development and `%LOCALAPPDATA%/com.cowworker.desktop/live` for release.
On macOS, Tauri resolves the directory under Application Support. The exact native path requires macOS validation.

`COWWORKER_DATA_DIR` overrides the workspace path only in debug builds. Automated native tests use a temporary directory.
On Windows, each workspace also uses its own `webview/` directory for browser preferences and cache.
The browser demo uses `cowworker.browser-demo.v1` in localStorage. Sidebar and theme preferences also use browser storage.

## Resources reported in the concept

| Resource | Intended role | Verification state |
| --- | --- | --- |
| Windows laptop and Mac Mini | Desktop development and validation | Reported by the user; toolchains not checked |
| Raspberry Pi 4 | Planned server host | Deployment not checked |
| cowworker.com on Cloudflare | Product domain | Ownership and configuration not checked |
| @cowworker_bot | Telegram notifications | Integration not checked |
| UptimeRobot and Resend | Monitoring and email | Configuration not checked |
| BUSY Bar and TourBox Neo | Optional hardware integrations | Device access not checked |

The concept lists BUSY Bar addresses 10.0.4.20 for USB and 10.0.0.120 for Wi-Fi.
Treat these as reported addresses until device access is checked.

## Planned environments

Development, native tests, and release data use separate local workspaces.
Server environments and deployment do not exist yet. No service credentials are necessary for the local workflow.

The document review date does not establish service or device availability.
See [stack](STACK.md) and [security](SECURITY.md).
