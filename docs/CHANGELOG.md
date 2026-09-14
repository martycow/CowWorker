---
owner: marty
created: '2026-09-13'
last_verified: '2026-09-13'
status: reference
---

# Changelog

This log records product changes.

## Record format

- The default version is `v0.1.0`.
- Each Major or Minor version change receives a brief summary of all changes since the previous version record.
- Each version record contains Date, Added Features, Removed Features, Improvements, and Fixed Bugs.

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
