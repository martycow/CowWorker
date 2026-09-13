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
Toolchain versions, installation steps, and build commands are not yet established.

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

Dev, Test, and Live are requested environment boundaries. Their configuration and deployment do not yet exist in this repository.
Separate data and credentials between environments. Use demonstration data for initial development.

The document review date does not establish service or device availability.
See [stack](STACK.md) and [security](SECURITY.md).
