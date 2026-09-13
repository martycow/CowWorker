---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: draft
---

# Security

This document separates implemented local controls from future security work. It makes no compliance claim.

## Implemented controls

- Rust validates desktop commands and writes parameterized SQLite queries.
- Source URLs permit HTTP and HTTPS. URLs with credentials are rejected.
- Document paths use generated UUIDs. Commands do not accept arbitrary filesystem paths.
- The Tauri window uses a content security policy and limited core permissions.
- React shows imported text without HTML interpretation.
- Submitted document versions remain immutable.
- The local workflow does not require credentials or transmit job search data to a server.
- Debug data and release data use separate directories.

Text imports have a 1 MB limit. Native tests use fictional content in a temporary workspace.
Source links open in the default browser only when the user selects them.
The native test enables a local debugging port on its own test process. Normal launches do not enable this port.

Workspace files are not encrypted by CowWorker. Local filesystem access can read the database and documents.

## Data to protect

CowWorker can contain resumes, contact details, employment history, private documents, and AI credentials.
The broader concept also includes financial and employment records. Their inclusion in the first release is undecided.

## Controls for future features

- Keep secrets out of source control, logs, and generated AI context.
- Use an operating-system credential store for provider keys.
- Limit Tauri commands and filesystem access to required operations.
- Treat imported documents, vacancies, and AI output as untrusted input.
- Require review before AI edits replace user facts or documents.
- Require explicit user action before external submission or messaging.
- Define authenticated synchronization and encrypted transport before deployment.

## Open decisions

Local encryption, backups, deletion, account recovery, retention, and server access controls require a concrete design.
Document parsers and external processes need defined limits and failure handling.

See [architecture](ARCHITECTURE.md) and [environment](ENVIRONMENT.md).
