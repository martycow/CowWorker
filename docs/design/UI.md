---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: accepted-direction
---

# User Interface

## Accepted direction

Sidebar that can be hidden and shown.

Graphite surfaces, lime primary actions, muted blue accents, readable vacancy rows, and contextual detail pane.
The interface uses English and a restrained cow identity.

## Sidebar

Hiding the sidebar must free space for the main content. Users must have a visible way to restore navigation.
The toggle appears at the top left of the main area. It remains visible when the sidebar is hidden.
The preference survives reloads. Narrow windows start with hidden navigation and close it after destination selection.
At widths up to 1050 CSS pixels, selecting a row opens its detail view with a Back to list action.

## Implemented visual system

The interface uses Segoe UI Variable and system fallbacks, with left-aligned text and compact rows.
The main colors are graphite `#111518`, surface `#171c20`, selection `#2b3943`, lime `#d3f394`, and muted blue `#b6ddf5`.
The selected prototype establishes this palette. A light theme uses the same hierarchy.

Vacancies use a semantic table and a contextual detail pane. Active controls support keyboard focus.
Dialogs keep focus inside, return focus to their trigger, and protect unsaved changes on dismissal.
Empty states provide a next action. Error states retain form input. Match analysis remains explicitly unavailable.

The supporting explorations are not independently approved designs.
Mock companies, requirements, and dates are demonstration data.

## Interaction checks

- Sidebar hiding and restoration work with a keyboard and preserve focus.
- Lists, details, and primary actions remain usable at narrow window widths.
- Loading, empty, error, and offline states explain the next action.
- Theme contrast and reduced-motion behavior receive a separate check.

Browser tests cover filters, sidebar state, dialog focus, narrow layouts, and the full workflow.
Native Windows tests cover real commands, exports, and persistence. The [verification record](../tech/VERIFICATION.md) defines their limits.

See [decision 003](../adr/001-025.md#decision-003-visual-direction-and-hideable-sidebar).

## Required extensions, not implemented

[Product](../business/PRODUCT.md) owns the requirements. [Development plan](../management/V0.2.0_DEVELOPMENT_PLAN.md) defines delivery and checks.

The shell adds a contextual AI panel, Universal Add, and a compact global task indicator.
The right AI panel also supports bottom placement. Hiding it must preserve a visible restore action and the main workspace.
The existing entity detail pane remains separate. At narrow widths, panels must not squeeze the workspace into unusable columns.
AI context follows page, entity, version, field, and selection. Navigation alone must not start model calls.

Inline suggestions and field comments show only useful context. Users can dismiss them and inspect evidence before applying proposals.
One AI Operation control uses the same marker everywhere. Its accessible name must explain the action without relying on the glyph.
The detail popover exposes purpose, provider/model, estimates when known, and optional credit information.
An AI marker never implies a paid action. Unknown cost stays explicitly unknown.

Universal Add accepts input before requiring classification. Review shows original input, extracted fields, confidence, and user corrections.
Users can accept selected candidates or retain overrides. Conflicts show the latest value and the proposal.
Long import work moves into Task Center. A review draft survives navigation and restart.
Task Center shows stage, progress, elapsed time, details, cancel, and retry with per-task state.
Completion toasts reuse the existing status pattern. Repeated progress updates must not flood assistive announcements.

Company cards show related-job counts and stage summaries. They never flatten all jobs into one company status.
Past/current employers receive distinct relationship badges. Missing logos reuse CompanyMark.
Company Details shows source/update information and explicit refresh.
AI Usage provides summary cards, tables, charts, filters, and history from the central ledger.
Charts need a text/table equivalent. Existing CSS tokens, themes, keyboard patterns, and semantic tables remain the visual foundation.
