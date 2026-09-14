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

Existing priorities remain provisional. New priorities follow the dependencies in the development plan.

## ID allocation

The [task registry](TASK_REGISTRY.md) stores the last issued number and every assigned ID.
All areas share one sequence. Task removal, completion, cancellation, or an area change never releases an ID.

Reserve each ID in the registry before adding the task here. Never calculate the next number from this active list.
Keep the full assigned ID when moving, editing, or reopening a task, even if its area changes.

## TODO

These tasks require a person, with optional AI assistance:

- [ ] [DEP-0001] [Med] [09-13-2026] Build and launch on macOS. The recorded validation had no macOS host.
- [X] [DEP-0002] [Med] [09-13-2026] Install and uninstall the Windows package on a clean machine.
- [ ] [DEP-0003] [Med] [09-13-2026] Validate signed packages and the update channel.
- [ ] [UI-0004] [Med] [09-13-2026] Validate assistive technology, contrast, and large real workspaces beyond the current test coverage.

## Current

No tasks are in development.

## Upcoming

- [ ] [IMP-0011] [High] [09-13-2026] P0: Reconcile baseline contracts and add ordered migrations for both schema-1 document variants.
- [ ] [FT-0012] [High] [09-13-2026] P1: Add immutable sources, field ownership, revisions, and reviewed proposals.
- [ ] [FT-0013] [High] [09-13-2026] P2: Add the persistent Rust task queue and global Task Center.
- [ ] [FT-0014] [High] [09-13-2026] P3: Add Company identity, relationships, legacy migration, and manual Company Hub.
- [ ] [FT-0015] [High] [09-13-2026] P5: Add Universal Import acquisition, processing adapters, and persistent review with FT-0006.
- [ ] [FT-0016] [Med] [09-13-2026] P6: Add source-backed company research, refresh, and managed logos.
- [ ] [UI-0017] [Med] [09-13-2026] P8: Add AI Usage queries, summaries, charts, and attempt history.
- [ ] [IMP-0018] [High] [09-13-2026] P9: Validate integrated workflows, migrations, recovery, and release evidence.
- [ ] [FT-0005] [Med] [09-13-2026] Add complete workspace backup and restoration with integrity checks.
- [ ] [FT-0006] [Med] [09-13-2026] Add PDF and DOCX import and export with parser limits and preview checks.
- [ ] [FT-0007] [Med] [09-13-2026] Add AI provider discovery and an evidence-based analysis contract.
- [ ] [FT-0008] [Med] [09-13-2026] Add a user-reviewed proposal flow for AI document changes.
- [ ] [PL-0009] [Med] [09-13-2026] Define synchronization identity, revisions, conflicts, and server boundaries.
- [ ] [FT-0010] [Med] [09-13-2026] After local scheduling rules are established, add calendar and notification adapters.

See [verification](../tech/VERIFICATION.md) for evidence and limits.

## Implementation sequence

The [development plan](V0.2.0_DEVELOPMENT_PLAN.md) is the detailed engineering handoff.
P0 uses IMP-0011 and the backup prerequisite from FT-0005.
P4 expands FT-0007 into the central AI runtime and ledger. P7 uses FT-0008 for contextual proposals.
FT-0006 retains import and export scope. PL-0009 and FT-0010 remain separate future work.
Phase labels are delivery groups, not replacement task IDs. No feature implementation started during this planning task.

See [backlog](BACKLOG.md) and [architecture](../tech/ARCHITECTURE.md).
