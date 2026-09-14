---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: implemented-baseline-and-planned-migrations
---

# Local data model

The first schema supports a saved vacancy, immutable document versions, one application per vacancy, and the next action.
Rust owns validation and all desktop data changes. Browser demo storage is independent and is not a SQLite implementation.

| Record | Relationships and rules |
| --- | --- |
| Vacancy | UUID; original description, source URL, role, company, location, work mode, notes, review state, timestamps |
| Document | UUID; title and fixed kind: resume, cover-letter, note, job-offer, agreement, tax-related, or other |
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

Compatibility finding: older schema-1 databases permit three document kinds. The current schema.sql permits seven but still sets user_version to 1.
Store::open does not reconcile that difference. A fresh database and an existing database can therefore accept different kinds.
P0 in the development plan must support both variants. The current documentation does not claim that this upgrade exists.

Version creation writes and flushes a new file before committing its metadata. A failed database write removes that new file.
A process crash between the file write and database commit can leave an unreferenced file; it must not be treated as a committed version.
Missing referenced files produce an explicit load error. The app does not silently replace them with empty content.

For a manual backup, close CowWorker and copy the entire workspace directory, including the database and documents.
Local encryption, automatic backups, multi-device revisions, conflicts, tombstones, and synchronization remain open work.

See [architecture](ARCHITECTURE.md) and [environment](ENVIRONMENT.md).

## Planned records and ownership

These records are not implemented. Detailed rules belong to [architecture](ARCHITECTURE.md).
Existing IDs, application history, submitted links, and document bytes must survive every migration.

| Record | Planned responsibility |
| --- | --- |
| SourceAsset / SourceSnapshot | Original bytes/text, hash, media type, acquisition time, URL, extraction tool/version, text/page anchors |
| EntitySource | Links one source to documents, vacancies, or companies without copying source bytes |
| FieldCandidate | Field key, raw value, normalized/generated value, source, confidence, run/operation, timestamps |
| FieldOverride | Explicit presence plus typed value, including null/empty values, author and revision |
| ChangeProposal | Target entity/base revision or document version, proposed fields/ranges, review state |
| BackgroundTask / Attempt | Status/progress, timestamps, error, cancellation, lease/generation, checkpoint, related entities |
| TaskDependency | Parent/child dependencies, required/optional stage, idempotency keys |
| Company | Independent UUID, identity, optional profile facts, user notes, revisions |
| CompanyAlias / Domain | Matching evidence and verified domain/alias association |
| EmploymentRelationship | Company ID, past/current relation, role, dates, user confirmation |
| AIOperation / AIAttempt | Logical action and actual model requests, model/provider, entity, status, latency, error |
| UsageMeasurement / PriceSnapshot | Nullable token counts, estimated/actual decimal cost, currency, price version, measurement quality |
| ImportSession / ImportItem / StageRun | Batch input, stage/version, classification/confidence, persisted review, target links |
| ResearchRun / CompanyAsset | Source-backed research and logo/image files, hashes, refresh dates |

Effective values remain in typed domain columns. An override wins over an accepted candidate.
All legacy populated values start protected. Manual changed-field commands update overrides and projections in one revision-checked transaction.
Generated results cannot overwrite manual edits, including deliberate empty values. Late acceptance must fail on revision conflict.
Document content proposals create new immutable versions. Category correction changes metadata without changing version bytes.
Original binary documents remain separate source assets. Extracted text is a derived representation, not the original binary.

Vacancy gains company_id and structured salary, employment type, requirements, responsibilities, stack, benefits, source metadata, and dates.
Salary retains currency, period, ranges, and the source text. Unknown values stay nullable.
Existing company strings remain historical snapshots. Application still references Vacancy with its current unique constraint.
Past/current employment is an explicit relationship, not an inferred application status.

## Proposed migration sequence

Version numbers are planning targets. Before implementation, read the current version and account for concurrent accepted migrations.
Application build version and SQLite schema version are separate counters.

| Target schema | Phase | Changes and upgrade gate |
| --- | --- | --- |
| 2 | P0 | Reconcile both schema-1 document constraints. Add ordered upgrades and tested backup/restore |
| 3 | P1 | Sources/provenance/proposals, revisions, structured vacancy fields, category expansion, metadata correction |
| 4 | P2 | Persistent tasks, attempts, dependencies, checkpoints, runner lease |
| 5 | P3 | Company identity, aliases/domains, employment relationships, nullable company_id and safe legacy backfill |
| 6 | P4 | AI operations/attempts, usage measurements, price snapshots, non-secret provider settings |
| 7 | P5 | Import sessions/items, stage runs, classification and review persistence |
| 8 | P6 | Company research runs and managed assets |
| 9 | P7 | Context preferences, annotation/dismissal records where proposal metadata is insufficient |
| 10 | P8 | Measured indexes for usage filters and aggregates |

Document categories expand without renaming existing values. Add job-description, portfolio, reference, and unknown.
Unknown means undecided classification. Other means the user accepted a category outside the named kinds.
Each migration must pass fresh-install, both legacy-upgrade, reopen, and foreign-key/file-integrity fixtures.
New fields have safe defaults. Reprocessing and backfill are idempotent and never infer user approval of researched facts.
The queue needs indexes on status/next-attempt and entity references. Company links and source hashes also need indexes.
File recovery reconciles temporary/unreferenced files without removing referenced assets. Backup covers database, documents, sources, and company assets.
