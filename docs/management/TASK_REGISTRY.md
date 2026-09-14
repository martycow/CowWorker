---
owner: marty
created: '2026-09-13'
last_verified: '2026-09-13'
status: active
---

# Task registry

This file is the authoritative counter and permanent record of assigned task IDs.
[Tasks](TASKS.md) contains active work. This registry retains each task after removal from that list.

## Counter

Last issued number: `0010`

All task areas share this counter. The next number is the last issued number plus one.
Numbers have at least four digits, with leading zeros. After `9999`, continue with `10000`.

## Allocation rules

1. Read the latest counter and registry entries before reserving a task ID.
2. Increase the counter by one and append the task entry in the same edit.
3. Save the registry before adding the task with that ID to `TASKS.md`.
4. Keep the assigned number and full ID permanently, including the original area prefix.
5. Never decrease the counter, reuse a number, fill a gap, or delete a registry entry.

The numeric part is unique across all areas. A different prefix does not permit the same number.
The counter must equal the highest number ever reserved, including completed, cancelled, and unused reservations.
An interrupted task creation leaves its reservation here. Resume that task with the same ID or record its cancellation.

## Completion and reopening

Before removing a task from `TASKS.md`, append its completion date and outcome to its registry entry.
For cancellation, append the date and reason instead. Keep the ID, task description, and earlier history.
Use `MM-DD-YYYY` for dates, as in `TASKS.md`.

If the same task reopens, append the reopening date and restore it to `TASKS.md` with its original ID.
A separate follow-up task receives a new ID. Priority, status, section, and area changes do not change an existing ID.

## Concurrent edits

Only one writer can reserve numbers at a time in the shared registry.
Before reserving IDs in separate branches, coordinate allocation through the shared registry.
During a merge, check numeric uniqueness, task references, and the counter against all retained reservations.

If separate branches assigned the same number to different tasks, stop integration and resolve the conflicting reservation.
Keep the accepted registry assignment. Give the conflicting unintegrated task a fresh number and update all its references together.
Never resolve a conflict by deleting an accepted entry or lowering the counter.

## Issued IDs

The initial entries normalize the ten active tasks to one global sequence.
Their earlier area-specific draft IDs had no references outside `TASKS.md` at initialization.
The IDs in this registry are permanent from initialization onward.

| ID       | Task                                                                                                 | Added      | Lifecycle history |
| -------- | ---------------------------------------------------------------------------------------------------- | ---------- | ----------------- |
| DEP-0001 | Build and launch on macOS.                                                                           | 09-13-2026 | Open              |
| DEP-0002 | Install and uninstall the Windows package on a clean machine.                                        | 09-13-2026 | Open              |
| DEP-0003 | Validate signed packages and the update channel.                                                     | 09-13-2026 | Open              |
| UI-0004  | Validate assistive technology, contrast, and large real workspaces beyond the current test coverage. | 09-13-2026 | Open              |
| FT-0005  | Add complete workspace backup and restoration with integrity checks.                                 | 09-13-2026 | Open              |
| FT-0006  | Add PDF and DOCX import and export with parser limits and preview checks.                            | 09-13-2026 | Open              |
| FT-0007  | Add AI provider discovery and an evidence-based analysis contract.                                   | 09-13-2026 | Open              |
| FT-0008  | Add a user-reviewed proposal flow for AI document changes.                                           | 09-13-2026 | Open              |
| PL-0009  | Define synchronization identity, revisions, conflicts, and server boundaries.                        | 09-13-2026 | Open              |
| FT-0010  | After local scheduling rules are established, add calendar and notification adapters.                | 09-13-2026 | Open              |
