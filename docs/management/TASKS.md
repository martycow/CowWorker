---
owner: marty
created: '2026-09-13'
last_verified: '2026-09-13'
status: active
---

# Tasks

This file lists planned tasks and tasks in development.
Before removing a completed task, record its outcome in the [permanent task registry](TASK_REGISTRY.md).

## Task areas

| Area        | Literal | Description                                        |
| ----------- | ------- | -------------------------------------------------- |
| Feature     | FT      | New business logic                                 |
| Improvement | IMP     | Changes to existing business logic                 |
| UI          | UI      | User interface and user experience                 |
| Planning    | PL      | Documentation, decisions, and brainstorming        |
| Deployment  | DEP     | Infrastructure and distribution setup              |
| Business    | BS      | Marketing, distribution, monetization, and metrics |

## Task format

| Part     | Description                                                                                          |
| -------- | ---------------------------------------------------------------------------------------------------- |
| Status   | `[ ]` = not started, `[!]` = in development, `[X]` = done before removal                             |
| ID       | Area literal and a global number from the task registry, separated by `-`. Use at least four digits. |
| Priority | `Low`, `Med`, `High`, or `BLOCKER`                                                                   |
| Added    | Date the task entered this list, in `MM-DD-YYYY` format                                              |

Format template (does not reserve an ID):

```text
- [!] [AREA-NNNN] [Low] [MM-DD-YYYY] Task description.
```

`[!]` is a text status marker. Markdown does not render it as a checkbox.

The existing tasks use `Med` as a provisional priority until review.

## ID allocation

The [task registry](TASK_REGISTRY.md) stores the last issued number and every assigned ID.
All areas share one sequence. Task removal, completion, cancellation, or an area change never releases an ID.

Reserve each ID in the registry before adding the task here. Never calculate the next number from this active list.
Keep the full assigned ID when moving, editing, or reopening a task, even if its area changes.

## TODO

These tasks require a person, with optional AI assistance:

- [ ] [DEP-0001] [Med] [09-13-2026] Build and launch on macOS. The recorded validation had no macOS host.
- [ ] [DEP-0002] [Med] [09-13-2026] Install and uninstall the Windows package on a clean machine.
- [ ] [DEP-0003] [Med] [09-13-2026] Validate signed packages and the update channel.
- [ ] [UI-0004] [Med] [09-13-2026] Validate assistive technology, contrast, and large real workspaces beyond the current test coverage.

## Current

No tasks are in development.

## Upcoming

- [ ] [FT-0005] [Med] [09-13-2026] Add complete workspace backup and restoration with integrity checks.
- [ ] [FT-0006] [Med] [09-13-2026] Add PDF and DOCX import and export with parser limits and preview checks.
- [ ] [FT-0007] [Med] [09-13-2026] Add AI provider discovery and an evidence-based analysis contract.
- [ ] [FT-0008] [Med] [09-13-2026] Add a user-reviewed proposal flow for AI document changes.
- [ ] [PL-0009] [Med] [09-13-2026] Define synchronization identity, revisions, conflicts, and server boundaries.
- [ ] [FT-0010] [Med] [09-13-2026] After local scheduling rules are established, add calendar and notification adapters.

See [verification](../tech/VERIFICATION.md) for evidence and limits.

See [backlog](BACKLOG.md) and [architecture](../tech/ARCHITECTURE.md).
