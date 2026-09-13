---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: active
---

# Tasks

## Completed documentation work

- [x] Record the application stack and storage direction.
- [x] Group ADRs by ranges of 25 and keep a link-only decision index.
- [x] Record the selected visual prototype and hideable sidebar.
- [x] Add metadata and initial content to all Markdown documents in docs.

## Completed implementation

- [x] Define the local data model and document version rules.
- [x] Select dependencies, npm, lockfiles, and build commands.
- [x] Create the Tauri, React, TypeScript, and Rust application structure.
- [x] Create the SQLite schema and transactional storage layer.
- [x] Implement vacancy capture, source preservation, editing, and duplicate URL checks.
- [x] Implement search, filters, sorting, shortlist, archive, and restoration.
- [x] Implement text documents, immutable versions, import, and export.
- [x] Implement application preparation, stages, history, and submitted document locking.
- [x] Implement next-action dates and the Today view.
- [x] Implement local profile facts and appearance preferences.
- [x] Implement sidebar hiding, restoration, keyboard focus, and narrow layouts.
- [x] Protect unsaved dialog changes and retain input after save errors.
- [x] Validate Rust state rules and persistence across database reopen.
- [x] Validate browser workflows, text export, and a 620-pixel layout.
- [x] Validate the Windows native workflow across process restart.
- [x] Build a Windows executable and an NSIS installer.
- [x] Add startup instructions and a verification record.

## Remaining validation

- [ ] Build and launch on macOS. No macOS host was available in this run.
- [ ] Install and uninstall the Windows package on a clean machine.
- [ ] Validate signed packages and the update channel.
- [ ] Check assistive technology, contrast, and large real workspaces beyond the current test coverage.

## Next implementation candidates

- [ ] Add complete workspace backup and restoration with integrity checks.
- [ ] Add PDF and DOCX import and export with parser limits and preview checks.
- [ ] Add AI provider discovery and an evidence-based analysis contract.
- [ ] Add a user-reviewed proposal flow for AI document changes.
- [ ] Define synchronization identity, revisions, conflicts, and server boundaries.
- [ ] Add calendar and notification adapters after local scheduling rules are established.

See [verification](../tech/VERIFICATION.md) for evidence and limits.

See [backlog](BACKLOG.md) and [architecture](../tech/ARCHITECTURE.md).
