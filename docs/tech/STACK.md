---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: accepted-direction
---

# Technology Stack

Status: Core stack implemented for the first local workflow.

Decision date: 2026-09-13.

## Accepted technologies

| Technology | Intended responsibility |
| --- | --- |
| Rust | Application logic, data access, and system integrations |
| React | User interface |
| TypeScript | Language for the React code |
| Tauri | Application shell and communication between React and Rust |
| SQLite | Local database on each device |

The user selected Rust, React, and Tauri.

Windows 11 and macOS remain the required desktop targets. Mobile integration and delivery require separate validation.

## Accepted storage direction

Each device stores application data in SQLite. Original documents remain separate files.

The planned application must support local work without an internet connection. A server will synchronize data between devices.

The synchronization protocol, conflict rules, and server implementation remain open decisions.

## Open decisions

- Backup and local encryption
- Synchronization protocol, conflict rules, and document transfer
- Document editor, import, and export libraries
- AI integration contracts and background execution
- Installation, signing, and updates
- Server responsibilities and mobile scope

See [the decision records](../DECISIONS.md) for the application stack and storage direction.

## Implemented build tools

The application uses npm, React 19, TypeScript 5.9, Vite 8, Tauri 2, and rusqlite 0.40 with bundled SQLite.
`package-lock.json` and `Cargo.lock` record exact resolved dependencies. Both files belong in source control.
Rust core tests cover persistence and state rules. Vitest covers list rules. Playwright covers browser and Windows WebView2 workflows.
The [environment document](ENVIRONMENT.md) contains verified tool versions and data locations.
