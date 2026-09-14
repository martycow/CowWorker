---
owner: marty
created: '2026-09-13'
last_verified: '2026-09-14'
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

The [implementation record](V0.2.0_IMPLEMENTATION.md) lists working behavior and remaining acceptance conditions for each task.
IMP-0011 and FT-0005 are complete; their outcomes remain in the permanent registry.
UI-0019 is complete: Settings displays the application version in its popover.

- [!] [FT-0012] [High] [09-13-2026] P1: Sources, ownership, revisions, and proposals implemented; finish source anchors and provenance inspection.
- [!] [FT-0013] [High] [09-13-2026] P2: Persistent queue and Task Center implemented; finish resource policies, graph admission, transient retries, and reconciliation.
- [!] [FT-0014] [High] [09-13-2026] P3: Company domain and manual Hub implemented; finish domain/alias matching and identity conflict review.
- [!] [FT-0007] [Med] [09-13-2026] P4: Runtime, model discovery, scope approval, and ledger implemented; finish capability/price/reconciliation contracts.
- [!] [FT-0015] [High] [09-13-2026] P5: Persistent import, Windows OCR, clipboard image handler, and ordered screenshot combination implemented; finish matching, stage reprocessing, and native acquisition matrix.
- [!] [FT-0006] [Med] [09-13-2026] PDF/DOCX extraction and export implemented; finish malformed/encrypted corpus and full UI acceptance.
- [!] [FT-0016] [Med] [09-13-2026] P6: Website refresh, proposals, and local logos implemented; finish broader research and remote logo adapters.
- [!] [FT-0008] [Med] [09-13-2026] P7: Contextual panel, guarded proposals, and vacancy-linked resume/cover letter preparation implemented; finish field/selection context and annotations.
- [!] [UI-0017] [Med] [09-13-2026] P8: Usage page and 10,000-attempt query fixture implemented; finish reconciliation, entity navigation, and accessibility coverage.
- [!] [IMP-0018] [High] [09-13-2026] P9: Core/browser/native recovery checks implemented; finish complete acceptance matrix and release/platform evidence.

## Upcoming

- [ ] [PL-0009] [Med] [09-13-2026] Define synchronization identity, revisions, conflicts, and server boundaries.
- [ ] [FT-0010] [Med] [09-13-2026] After local scheduling rules are established, add calendar and notification adapters.

See [verification](../tech/VERIFICATION.md) for evidence and limits.

## Implementation sequence

The [development plan](V0.2.0_DEVELOPMENT_PLAN.md) is the detailed engineering handoff.
P0 uses IMP-0011 and the backup prerequisite from FT-0005.
P4 expands FT-0007 into the central AI runtime and ledger. P7 uses FT-0008 for contextual proposals.
FT-0006 retains import and export scope. PL-0009 and FT-0010 remain separate future work.
Phase labels are delivery groups, not replacement task IDs. Implementation is in progress; the original plan remains the acceptance baseline.

See [backlog](BACKLOG.md) and [architecture](../tech/ARCHITECTURE.md).
