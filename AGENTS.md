# AGENTS.md - CowWorker

## What this project is

CowWorker is a multiplatform app that improves users' job hunting process. It has many built-in features which assist user on every step related to job, from writing resume to signing work contract.

## Development

- Read `README.md` for startup and checks, `docs/tech/DATA_MODEL.md` for storage rules, and `docs/management/TASKS.md` for current work.
- Keep desktop data changes in the Rust library under `backend/`. The Tauri shell is under `src-tauri/`; React sources are under `frontend/`.
- Commit `Cargo.lock` and `package-lock.json`. Keep development, test, and release data separate.
- Run `npm.cmd run check` for relevant changes. Run `npm.cmd run test:e2e` for workflow changes.
- For native changes, build with `npm.cmd run tauri -- build --debug --no-bundle` and run `npm.cmd run test:native` on Windows.
- Browser checks do not establish native behavior or macOS support. Record these evidence boundaries in `docs/tech/VERIFICATION.md`.
- Preserve immutable submitted document versions. Do not transmit data to employers or services without explicit authorization.

## Decision record layout

- Keep only links to individual decisions in `docs/DECISIONS.md`.
- Store decision records in `docs/adr/`, grouped by ranges of 25 decision numbers.
- Use `001-025.md`, `026-050.md`, `051-075.md`, and subsequent ranges. The current file can contain fewer than 25 decisions.
- Give each decision a numbered heading and link to that heading from `docs/DECISIONS.md`.
