---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: implemented-baseline
---

# Architecture

This document describes the first local implementation and the remaining architecture decisions.

## Accepted boundaries

React and TypeScript provide the UI. Tauri connects the UI to Rust application logic and system integrations.
Each device uses SQLite and separate original document files. A server synchronizes data between devices.
Local work must remain available without server access.

See [decisions 001 and 002](../adr/001-025.md).

## Implemented structure

`frontend/` contains the React interface, typed command adapter, forms, and local list rules.
`src-tauri/` contains the desktop shell and explicit commands. `backend/` contains a separate Rust library for storage and validation.

Rust owns desktop data changes. SQLite stores relationships and metadata. Separate UTF-8 files store immutable document versions.
Commands serialize database access through a mutex. Application changes and history events share a transaction.
The interface keeps failed forms open and shows an error. Database startup errors appear inside the app and support a retry.

The browser adapter uses a separate localStorage namespace for demonstration. It does not replace native persistence tests.
There are no AI providers, background workers, or server modules yet.

## Open decisions

- Migrations after schema 1 and a backup interface
- Synchronization protocol, conflict rules, deletions, and document transfer
- Background execution when the window or application is closed
- AI context files and their relationship to stored application data
- Authentication, server stack, and mobile architecture

See [data model](DATA_MODEL.md) for current identifiers, states, file layout, and version rules.
No server or synchronization protocol is implemented.
