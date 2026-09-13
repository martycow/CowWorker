---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: accepted-direction
---

# Technology Stack

Status: Accepted core stack. Implementation has not started.

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

- Dependency versions and package manager
- Database schema, file layout, backup, and encryption
- Synchronization protocol, conflict rules, and document transfer
- Document editor, import, and export libraries
- AI integration contracts and background execution
- Installation, signing, and updates
- Server responsibilities and mobile scope

See [the decision records](../DECISIONS.md) for the application stack and storage direction.
