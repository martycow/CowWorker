---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: implemented
---

# Local data model

The first schema supports a saved vacancy, immutable document versions, one application per vacancy, and the next action.
Rust owns validation and all desktop data changes. Browser demo storage is independent and is not a SQLite implementation.

| Record | Relationships and rules |
| --- | --- |
| Vacancy | UUID; original description, source URL, role, company, location, work mode, notes, review state, timestamps |
| Document | UUID; title and fixed kind: resume, cover letter, or note |
| Document version | UUID; document ID, increasing version number, immutable UTF-8 text file, creation time |
| Application | UUID; unique vacancy ID, stage, notes, next action, optional calendar date, submission and change times |
| Application document | Links an application to an exact document version |
| Application event | Append-only stage event, written in the same transaction as the stage change |
| Profile | Name, email, headline, and user-written career facts; one local profile |

## Vacancy states

`saved` means awaiting review. A vacancy can become `reviewed` or `shortlisted`.
`archived` hides it from active lists. Restoration returns it to `saved`.
Archiving preserves its source, notes, applications, and documents.

HTTP and HTTPS source URLs are normalized and fragments are removed. Credentials and other URL schemes are rejected.
Nonempty source URLs are unique. Tracking parameters remain intact; deduplication does not claim that distinct URLs describe distinct jobs.

## Application stages

Stages are `preparing`, `applied`, `interview`, `offer`, `rejected`, and `withdrawn`.
Preparing a vacancy twice returns the same application.
The initial version intentionally permits stage corrections and skipped stages, except returning to `preparing` after submission.

The first transition to applied, interview, or offer records the submission time and requires at least one attached document version.
This records a submission made outside CowWorker. It does not contact an employer.
Submitted document attachments cannot change. Later edits create new document versions and leave submitted content intact.
There is no requirement to attach a resume specifically: an application may have been sent with a cover letter or another text document.

Offer, rejected, and withdrawn applications do not contribute to active reminders.
A due date requires a next action. Dates use `YYYY-MM-DD`; timestamps use RFC 3339 in UTC.
The Today page compares dates in the device's local calendar. This version does not deliver background notifications.

## Storage and migrations

`cowworker.db` holds metadata and relationships. `documents/<version-uuid>.txt` holds each exact saved text version.
Files use generated UUID names, never a user-provided path. The UI supports UTF-8 `.txt` and `.md` imports up to 1 MB.
The import retains the decoded text, including line endings; it is not a binary document importer.

SQLite enables foreign keys, WAL, and a five-second busy timeout. Schema creation runs in a transaction; `PRAGMA user_version` is 1.
A newer schema version is refused. Add ordered migrations before changing this schema.

Version creation writes and flushes a new file before committing its metadata. A failed database write removes that new file.
A process crash between the file write and database commit can leave an unreferenced file; it must not be treated as a committed version.
Missing referenced files produce an explicit load error. The app does not silently replace them with empty content.

For a manual backup, close CowWorker and copy the entire workspace directory, including the database and documents.
Local encryption, automatic backups, multi-device revisions, conflicts, tombstones, and synchronization remain open work.

See [architecture](ARCHITECTURE.md) and [environment](ENVIRONMENT.md).
