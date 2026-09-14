# CowWorker

CowWorker is a local job search workspace built with Rust, React, TypeScript, Tauri, and SQLite.

By default, it supports vacancy capture, document versions, application history, next actions, and a local career profile.

## Start on Windows

Install Node.js 22.12 or later, Rust, Visual Studio C++ Build Tools, and WebView2.
Then run these commands from the repository root in PowerShell:

| Command                      | Purpose                                                    |
|------------------------------|------------------------------------------------------------|
| `npm.cmd ci`                 | Install the locked JavaScript dependencies                 |
| `npm.cmd run tauri -- dev`   | Start the desktop app with development data                |
| `npm.cmd run dev`            | Start the separate browser demo at `http://127.0.0.1:1420` |
| `npm.cmd run tauri -- build` | Build the release executable and Windows installer         |

Use `npm` instead of `npm.cmd` on macOS. macOS also requires Xcode Command Line Tools.
The [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) describe the platform requirements.

The Windows installer appears in `target/release/bundle/nsis/`.
The standalone executable appears at `target/release/cowworker.exe` and requires WebView2.

## First workflow

1. Add a vacancy and paste its original description.
2. Create or import a resume under Documents.
3. Select Prepare application on the vacancy.
4. Attach the exact document versions under Update application.
5. After you submit externally, select Applied and save the application.
6. Set a next action and a date. Due actions appear under Today.

CowWorker does not contact employers. The Applied stage records your external submission.
Later document edits create new versions. Submitted applications keep the earlier versions.

## Checks

| Command                                                 | Purpose                                                         |
|---------------------------------------------------------|-----------------------------------------------------------------|
| `npm.cmd run check`                                     | Build the interface and run frontend and Rust core tests        |
| `npm.cmd run test:e2e`                                  | Run browser workflow checks with Microsoft Edge                 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Check Rust code and reject warnings                             |
| `cargo fmt --all --check`                               | Check Rust formatting                                           |
| `npm.cmd run format:check`                              | Check frontend and test formatting                              |
| `npm.cmd run tauri -- build --debug --no-bundle`        | Build the native test executable                                |
| `npm.cmd run test:native`                               | Run the Windows workflow against real Tauri commands and SQLite |

The native test creates a temporary workspace. Its report and screenshots appear under `output/verification/`.
It checks persistence across process restarts and the content of exported document versions.

## Data and scope

Development and release builds use separate directories under the Tauri local application directory.
On Windows, these directories are `%LOCALAPPDATA%/com.cowworker.desktop/dev` and `%LOCALAPPDATA%/com.cowworker.desktop/live`.
Each workspace contains `cowworker.db` and `documents/`. Close CowWorker before you copy the entire directory for a manual backup.

The browser demo uses separate browser storage. It does not read or modify the desktop workspace.
Fictional examples are available only through the browser demo action.

## Project context

[Product](docs/business/PRODUCT.md) owns requirements. [Architecture](docs/tech/ARCHITECTURE.md) owns boundaries and planned contracts.
[Data model](docs/tech/DATA_MODEL.md) distinguishes the current schema from planned migrations.
[Development plan](docs/management/V0.2.0_DEVELOPMENT_PLAN.md) contains the repository audit, dependencies, phases, and acceptance criteria.
[Tasks](docs/management/TASKS.md) tracks active work. [Verification](docs/tech/VERIFICATION.md) separates recorded results from current evidence.

Contextual AI, Universal Import, Companies, Background Tasks, and AI Usage are required extensions. They are not implemented yet.
The plan records a current TypeScript error and a schema compatibility risk before feature work starts.
