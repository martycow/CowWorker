---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-14"
status: implemented-baseline-and-planned-contracts
---

# Architecture

This file owns technical boundaries and target contracts. [Product](../business/PRODUCT.md) owns requirements.
The [development plan](../management/V0.2.0_DEVELOPMENT_PLAN.md) owns implementation steps.
The [implementation record](../management/V0.2.0_IMPLEMENTATION.md) separates current behavior from remaining target contracts.

## Implemented boundaries

React and TypeScript provide the UI. `frontend/App.tsx` holds page state, workspace data, selection, forms, and mutation state.
`frontend/api.ts` invokes explicit Tauri commands or uses isolated browser localStorage.
`src-tauri/src/main.rs` owns startup, system URL opening, and `Mutex<Option<Store>>`.
`backend/src/store.rs` owns validation, SQLite operations, and immutable UTF-8 document files.
Application updates and history events share transactions. SQLite uses WAL, foreign keys, and a busy timeout.

The UI reloads workspace metadata after mutations. Document content loads separately for an explicitly selected version.
The global `act`/`busyRef` guard permits one UI mutation at a time. It is not a scheduler.
Universal Add retains original sources and persistent review drafts. Explicitly authorized public URL imports use a bounded Rust HTTP adapter.
Windows image and scanned PDF OCR run in the isolated parser process. Combined imports retain source order and each original file.
Prepared resumes and cover letters link to a vacancy and an immutable original resume version. The AI preview includes these related records.
The Rust queue owns execution independently of navigation. One worker processes tasks; a separate heartbeat renews its lease.
The AI runtime persists logical operations, provider attempts, authorization scopes, and nullable usage measurements.
Company records own manual facts, employment relationships, source proposals, and managed logos. Application stages remain on applications.
Task Center creates verified backups and restores separate workspace copies. Storage connections close before the active path changes.
Server synchronization remains a planned direction from decision 002.

## Target module boundaries

The following modules exist, except `frontend/shell/`. App.tsx still owns the shell. The implementation record lists incomplete contracts within each module:

| Boundary | Responsibility |
| --- | --- |
| `backend/src/migrations/` | Ordered migrations and compatibility |
| `backend/src/provenance.rs`, `sources.rs` | Sources, field candidates, overrides, revisions, review commits |
| `backend/src/tasks/` | Durable queue, dependencies, attempts, cancellation, recovery |
| `backend/src/ai/` | Operation entry point, provider adapters, authorization, usage ledger |
| `backend/src/import/` | Acquisition, extraction, classification, parsing, normalization, drafts |
| `backend/src/companies/` | Identity, matching, relationships, research, assets |
| `src-tauri/src/` | Runner startup, short commands, events, OS file/credential adapters |
| `frontend/shell/`, `tasks/`, `ai/`, `import/`, `companies/` | Shared shell services and feature presentation |

The Rust core stays independent of Tauri. ModelProvider abstracts model transport; the current implementation directly uses keyring and the system clock.
Existing Store methods remain reusable. Modules move only when a phase requires a boundary.
The browser adapter remains a deterministic demo/test adapter. It does not own native tasks or provider credentials.

## Background tasks

Rust owns the queue in workspace SQLite. Tauri starts one runner after successful workspace initialization.
Enqueue commands commit task records and return IDs promptly. Network and extraction work runs outside database locks.
Short Store transactions claim tasks and commit results. A workspace runner lease prevents duplicate execution across app processes.
Claim generations fence late results from expired workers. The current mutex alone cannot protect separate processes.

Tasks persist ID, type/version, status, progress, status text, timestamps, error, cancellability, related IDs, input references, and revision.
Attempts retain start/end, retry reason, lease, checkpoint, and next eligible time.
Dependency edges form a bounded acyclic graph. Parents distinguish optional enrichment errors from required failures.
States are queued, running, waiting, completed, failed, and cancelled.
`waitingReason` distinguishes review, authorization, connectivity, and retry delay.

Initial policy uses two network slots, one extraction slot, and one concurrent call per provider.
These are tunable defaults, not measured throughput claims. A full worker pool does not prevent batch admission.
Safe transient failures receive at most three total attempts with exponential backoff and jitter. Provider retry hints take precedence.
Invalid input and denied authorization require user action. Unknown remote outcomes do not trigger blind model retries.

Cancellation prevents new child work and requests cooperative cancellation of active work.
Bounded parser subprocesses can be terminated. An in-flight model call can still finish or incur usage.
Commit transactions check cancellation and worker generation. Already committed records remain saved.
Completed stages have idempotency keys. Retries reuse results instead of creating duplicate domain records.

Navigation and window visibility do not own execution. Process exit stops execution.
On restart, the runner reconciles interrupted tasks and resumes safe stages from checkpoints.
Unknown provider outcomes wait for reconciliation or an explicit retry decision.
There is no OS service or change to tray/window lifetime in this plan.

The shell subscribes before loading persisted snapshots, compares revisions, and reloads after reconnect or focus.
Events contain IDs and revisions, not documents or secrets. Event loss cannot lose task state.
A channel is optional for bounded live output, not persistence.
This transport choice follows [Tauri event/channel guidance](https://v2.tauri.app/develop/calling-frontend/).
[Tauri commands](https://v2.tauri.app/develop/calling-rust/) remain the request boundary.

## Field ownership

Original bytes/text remain immutable assets with hash, media type, acquisition time, and source URL where available.
Source snapshots record extraction tool/version and text or page coordinates. Each fetch creates a new snapshot.
Candidates record entity/field, raw extracted value, normalized/generated value, source reference, confidence, operation/run, and timestamps.
Overrides record value and explicit presence. An empty string or null override is not absence.
Effective value is the override when present, otherwise the accepted candidate.

Existing domain columns remain effective projections for reads/searches. Typed validators control writes.
Manual edits update overrides and projections atomically. Legacy populated fields start protected as user-owned.
Candidates never update protected fields. Review commits selected candidates with an expected entity revision.
New commands carry changed fields and base revision. Unchanged form values must not become accidental overrides.
Stale proposals return a conflict with current values. Replacing an override requires an explicit replacement action.
Clearing an override is explicit and reveals the accepted candidate. Refresh cannot clear it.

Document content uses immutable versions instead of generic per-character field storage.
Suggestions target a base version and text-range fingerprint. Changed text makes suggestions stale.
Acceptance creates a new version. Submitted references remain unchanged.
Category corrections update document metadata with revision checks, without changing version bytes or attachments.

## Universal Import

Acquisition adapters accept URL, file, paste, and drop without requiring an entity choice.
Rust stores input before long processing. File picker/drop adapters grant access only to selected inputs.
Acquisition copies bounded data into managed storage. Later frontend calls use source IDs, not large base64 payloads.
Extractors produce text and layout/source references. OCR is a distinct stage.
Deterministic parsing does not pretend to be an AI Operation.

Classification suggests entity/category with confidence and method. Ambiguous input goes to review.
Parsing produces typed candidates. Normalization preserves raw values, currencies, salary periods, and source dates.
Optional enrichment uses the AI runtime. Manual/text capture does not require a model.
Review persists as a draft across restart. Save creates domain records and source links in one idempotent transaction.
Existing vacancy validation, application preparation, and document version creation remain domain services.

URL acquisition validates HTTP(S), redirects, DNS/private addresses, timeout, and byte limits.
Authenticated or script-only pages fall back to paste/upload. Parsing cannot execute page scripts or document macros.
PDF/DOCX/image adapters enforce decompression, page, pixel, memory, and execution limits.
Parser selection requires a Windows/macOS fixture spike before dependencies become mandatory.

## Companies

Company owns identity, optional profile facts, aliases/domains, source-backed research, and assets.
Vacancy gains `company_id`. Existing company text remains a historical display snapshot during migration.
Applications still reference vacancies. The unique application-per-vacancy constraint remains.
Employment relationships are explicit records with role and dates. Application stages remain per application.

Explicit company selection takes precedence. Verified company domains and aliases provide strong matching evidence.
Job-board domains do not identify employers. A normalized name alone suggests candidates but does not silently merge them.
Migration groups exact trimmed legacy company labels as provisional identities and retains original labels and vacancy IDs.
Case-only and fuzzy matches remain review candidates. Users can split provisional groups or merge identities through transactional relinking.
Merge preserves aliases, sources, relationships, and history. Conflicting user values require review.

Research and logo acquisition run as independent tasks after core records are saved.
Facts retain source URLs, observation dates, confidence, and last successful refresh.
An initial 30-day stale indicator is configurable. It does not trigger automatic network work.
Failed refresh preserves prior facts and images. Failed logos use the existing CompanyMark fallback.
Assets use managed files with hashes and source metadata.

## AI runtime and accounting

One Rust operation service owns every model call, including classification, model-based OCR, and research summarization.
Requests contain operation type, entity/base revision, source IDs, authorization scope, provider/model, and idempotency key.
Provider adapters declare capabilities and return typed results and usage metadata.
OS adapters store credential references. Workspace rows, frontend state, logs, and model context never contain provider keys.
Remote authorization covers data and purpose. Expanded data or new destinations require a new authorization scope.
Imported text is evidence, not instructions or permission to call tools.

Logical operations have separate provider attempts. Attempts record provider/model, status, latency, errors, request ID, and usage quality.
Input/output/total tokens can be unknown. Estimated and actual costs remain separate, with currency and a price snapshot.
Amounts use integer minor subdivisions or exact decimals. Different currencies are not summed without a recorded conversion.
Duplicate callbacks update one attempt. Retries create attempts and preserve prior billable usage.
Failed, interrupted, and cancelled attempts remain visible. Unknown usage is never converted into zero cost.
Completed model output can remain saved while its proposal waits for review.

AI Usage reads paginated ledger queries and grouped aggregates. Features never maintain separate counters.
Local/BYOK/free/subscription modes use the same operation identity. Credit balance remains optional metadata.
No provider, billing model, or external research service is selected by this planning task.

## Shell and scaling

Page state remains sufficient for initial navigation. These changes do not require a router or state library by default.
Shell task and AI context stores use typed subscriptions and independent pending states.
AI context contains page, entity ID, base revision, document version, field, and selection.
Navigation does not change an existing operation target or start another operation.
Task, usage, and company queries are paginated. Document bodies load on demand before large binary imports are enabled.
Unsaved form drafts survive background invalidation. Entity details and AI panels share responsive space without replacing each other.

See [data model](DATA_MODEL.md) for migrations and [decision 005](../adr/001-025.md#decision-005-contextual-workspace-and-planned-runtime-boundaries) for status.
