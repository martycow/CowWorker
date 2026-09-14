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
