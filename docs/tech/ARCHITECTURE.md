---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: draft
---

# Architecture

This document separates accepted boundaries from a proposed implementation structure.

## Accepted boundaries

React and TypeScript provide the UI. Tauri connects the UI to Rust application logic and system integrations.
Each device uses SQLite and separate original document files. A server synchronizes data between devices.
Local work must remain available without server access.

See [decisions 001 and 002](../adr/001-025.md).

## Proposed structure

Use one application with modules for Profile, Vacancies, Applications, Documents, AI, and Integrations.
Rust services own data changes. UI components request operations through explicit Tauri commands.
Keep provider-specific behavior behind adapters.

Store metadata and document version relationships in SQLite. Keep original files and submitted copies addressable from those records.
Represent long-running work with explicit state, cancellation, and error handling.

## Open decisions

- Schema, identifiers, migrations, and file layout
- Synchronization protocol, conflict rules, deletions, and document transfer
- Background execution when the window or application is closed
- AI context files and their relationship to stored application data
- Authentication, server stack, and mobile architecture

No module structure, server, or synchronization protocol is implemented.
