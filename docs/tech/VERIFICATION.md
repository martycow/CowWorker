---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: verified-windows
---

# Verification of the first local workflow

## Local results

| Check | Result | Scope |
| --- | --- | --- |
| TypeScript and Vite build | Passed | Production frontend assets |
| Vitest | 5 passed | Combined vacancy queries, ordering, calendar dates, active application rules |
| Rust core tests | 6 passed | SQLite, document files, duplicate URLs, input errors, transactional submission, missing files, future schema rejection |
| Browser workflow tests | 7 passed | Vacancy to application, version history, import/export, sidebar, theme, profile, dialog focus, unsaved changes, 620-pixel layout |
| Rust Clippy | Passed with warnings denied | Workspace and test targets |
| Windows debug build | Passed | Native Tauri executable |
| Windows native workflow | Passed | Real Tauri IPC, SQLite, text export, and process restart |
| Windows release build | Passed | Executable and NSIS installer |
| Windows release launch | Passed | Real window with an empty local workspace and no test records |

## Native test

`scripts/native-smoke.mjs` launches the debug executable with a temporary data directory.
Playwright connects to that process through a local WebView2 debugging endpoint.
The test creates a vacancy, saves a resume, exports its text, and prepares an application.
It records a submission with version 1, then creates version 2.
It stops the process and starts it again. The application still references version 1, and Today still shows the due action.

The report is `output/verification/native-report.json`. Screenshots are in the same directory.
The test keeps its temporary data for diagnosis. It does not modify the release workspace.
The test isolates the WebView2 profile as well as the database and document files.
Separate Windows UI automation also inspected the real window and exercised the sidebar toggle.

## Evidence limits

The native automated check uses a debug executable with bundled frontend assets. It does not exercise the installer or the update mechanism.
The release executable also starts successfully. Clean-machine installation and uninstallation remain open checks.
No macOS native build or launch was available. Mobile targets are not implemented.
A Windows and macOS CI workflow is configured in `.github/workflows/build.yml`. It was not run remotely during this session.
The tests do not establish comprehensive accessibility, security, crash recovery, or large-workspace performance.
External source opening uses the system browser adapter. The native smoke test does not open a third-party page.

## Reproduction

The [README](../../README.md) lists the build and test commands.
Tauri documents the [Vite build configuration](https://v2.tauri.app/start/frontend/vite/) and [desktop test options](https://v2.tauri.app/develop/tests/).
