---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: accepted-direction
---

# User Interface

## Accepted direction

The user selected the [first prototype](prototypes/2026-09-13/01-vacancies.png) with a sidebar that can be hidden and shown.
The image remains the visual reference; it does not yet depict the requested hidden state.

Retain its graphite surfaces, lime primary actions, muted blue accents, readable vacancy rows, and contextual detail pane.
The interface uses English and a restrained cow identity.

## Sidebar

Hiding the sidebar must free space for the main content. Users must have a visible way to restore navigation.
The exact toggle placement, narrow-window behavior, and persistence of the preference remain implementation details to resolve.

## Supporting references

- [Original style reference](../screenshots/Style_Reference.jpg)
- [Application detail exploration](prototypes/2026-09-13/02-application.png)
- [Resume editor exploration](prototypes/2026-09-13/03-resume-editor.png)

The supporting explorations are not independently approved designs.
Mock companies, requirements, and dates are demonstration data.

## Proposed interaction checks

- Sidebar hiding and restoration work with a keyboard and preserve focus.
- Lists, details, and primary actions remain usable at narrow window widths.
- Loading, empty, error, and offline states explain the next action.
- Theme contrast and reduced-motion behavior receive a separate check.

These checks describe future validation. The images do not prove working interactions.

See [decision 003](../adr/001-025.md#decision-003-visual-direction-and-hideable-sidebar).
