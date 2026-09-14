---
owner: marty
created: '2026-09-13'
last_verified: '2026-09-14'
status: reference
---

# Changelog

This log records product changes.

## Record format

- The default version is `v0.1.0`.
- Each Major or Minor version change receives a brief summary of all changes since the previous version record.
- Each version record contains Date, Added Features, Removed Features, Improvements, and Fixed Bugs.

## v0.2.0

### Date

2026-09-14. Local Windows build v0.2.15. This version is not a published release.

### Added Features

- Added original source storage, persistent import review, and ordered screenshot combination.
- Added PDF/DOCX resume import, local Windows screenshot OCR, scanned PDF OCR, and PDF/DOCX export.
- Added vacancy-linked resume copies and cover letter templates with immutable version history.
- Added contextual AI previews and reviewed proposals using the selected resume, vacancy, and profile.
- Added the Rust AI runtime, compatible HTTP adapter, model discovery, protected credentials, and central AI Usage ledger.
- Added the persistent task queue, Task Center, Company Hub, website metadata refresh, and local company logos.
- Added verified workspace backup, separate restore copies, and automatic backups before schema upgrades.
- Added the application version to the Settings popover.

### Removed Features

None recorded.

### Improvements

- Added ordered schema upgrades through version 9 and explicit recovery of interrupted tasks.
- Preserved exact original source files and submitted document versions across edits and restarts.
- Added a complete native resume-to-PDF workflow using fictional inputs and a loopback model fixture.
- Added a [document preparation guide](guides/APPLICATION_DOCUMENTS.md) and detailed [verification evidence](tech/VERIFICATION.md).
- Kept incomplete phase requirements visible in the [implementation record](management/V0.2.0_IMPLEMENTATION.md).

### Fixed Bugs

- Fixed work-mode spelling and both legacy document-category constraints.
- Fixed long-paragraph PDF wrapping and HTML entity handling in structured vacancy extraction.
- Fixed the file input overlay that intercepted other import controls.
- Serialized startup storage initialization and added record-preservation guards to schema upgrades.
- Prevented repeated model dispatch after an interrupted request with an unknown remote outcome.

## v0.1.0

### Date

2026-09-13. Local development build. This version is not a published release.

### Added Features

- Added the Rust storage library, SQLite schema, and immutable text document versions.
- Added the Tauri desktop app and the React interface based on the selected visual direction.
- Added vacancy capture, editing, search, filters, shortlist, archive, and restoration.
- Added document editing, UTF-8 text import, version history, and text export.
- Added applications, locked submitted versions, stage history, and next-action dates.
- Added Today, a local profile, theme selection, and sidebar preferences.

### Removed Features

None recorded.

### Improvements

- Added browser and native workflow tests, including persistence across process restarts.
- Built the Windows executable and NSIS installer.
- Recorded Rust, React, TypeScript, Tauri, and SQLite as the accepted stack.
- Recorded local document files and server synchronization as the storage direction.
- Created three visual window prototypes.
- Selected the first prototype with a hideable sidebar.
- Organized decisions into a link-only index and ADR files with ranges of 25.
- Added frontmatter and initial content to all Markdown documents in `docs/`.

### Fixed Bugs

None recorded.

## Project history

- 2026-09-12: The original concept records this date as its creation date.

See [decisions](DECISIONS.md) for accepted choices.
