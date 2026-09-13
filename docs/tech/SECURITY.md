---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: draft
---

# Security

This is a proposed baseline. No security controls or compliance claims are verified.

## Data to protect

CowWorker can contain resumes, contact details, employment history, private documents, and AI credentials.
The broader concept also includes financial and employment records. Their inclusion in the first release is undecided.

## Proposed controls

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
