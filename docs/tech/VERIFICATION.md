---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: verified-windows
---

# Verification of the first local workflow

## Historical local results

These results describe the first local workflow session. They are not a fresh run against every later commit.

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
The user subsequently marked DEP-0002 done in TASKS.md. This audit preserved that mark without repeating the installer check.
The recorded native smoke evidence does not cover that separate user-reported completion.
No macOS native build or launch was available. Mobile targets are not implemented.
A Windows and macOS CI workflow is configured in `.github/workflows/build.yml`. It was not run remotely during this session.
The tests do not establish comprehensive accessibility, security, crash recovery, or large-workspace performance.
External source opening uses the system browser adapter. The native smoke test does not open a third-party page.

## Reproduction

The [README](../../README.md) lists the build and test commands.
Tauri documents the [Vite build configuration](https://v2.tauri.app/start/frontend/vite/) and [desktop test options](https://v2.tauri.app/develop/tests/).

## Context and planning audit — 2026-09-13

Baseline: c75ff7ae4cdaff7fbe88fc852e91886c328513d4, plus the user's existing DEP-0002 completion mark.
The audit inspected frontend, Rust core, Tauri commands, schema, manifests, design tokens, tests, and agent/product/planning documents.
The new [development plan](../management/V0.2.0_DEVELOPMENT_PLAN.md) identifies current code and planned modules separately.

The current TypeScript check reports TS2820 at frontend/api.ts:195.
The demo uses On-site, while WorkMode declares On-Site. SQL, forms, and Rust validation use On-site.
This audit records the error for P0. It does not fix production code.

Source inspection also found different document-kind constraints under schema version 1 and no ordered migration runner.
P0 requires fixtures for both variants. No live user database was opened or migrated during this audit.

Documentation validation passed for 21 Markdown files, 71 local links/anchors, and 18 unique registered active task IDs.
The counter is 0018. The existing DEP-0002 completion mark remains intact.
All 17 changed/new paths are documentation. No production paths changed, and git diff --check passed.
No frontend build, end-to-end suite, native build, or native workflow ran for this documentation-only task.
Application versions and lockfiles remain unchanged. Planned systems have no runtime verification yet.
