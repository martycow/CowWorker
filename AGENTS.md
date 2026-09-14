# AGENTS.md - CowWorker

## What this project is

CowWorker is a multiplatform app that improves user's job hunting process. It has many built-in features which assist
user on every step related to job, from writing resume to signing work contract, and career development.

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

## Task IDs

- Use `docs/management/TASK_REGISTRY.md` as the authoritative task counter and permanent ID registry.
- Reserve the next global number in the registry before adding a task to `docs/management/TASKS.md`.
- Use one sequence across all task areas. Never decrease the counter, reuse a number, or change an assigned ID.
- When a task is completed or cancelled, record its outcome in the registry before removing it from `TASKS.md`.
- Keep registry entries permanently. Follow the registry rules for concurrent edits and merge conflicts.

## Version management

Application has current version to better compare it with older states. 

- Versioning starts with v0.1.0
- The format is "v{Major}.{Minor}.{Build}"
- Major version will rarely be changed. Until public release it equals 0. Major version 1 means that application was publicly released. Increase Major version if the whole product changed significantly since previous Major version.
- Minor version is an intermediate version. When Major version changes, Minor version resets to 0. Minor version increased when there are big and noticeable changes.
- Build version is increased any time build process succeed. When Major or Minor version change, Build version resets to 0.
- There are multiple files in the project which use versioning for their own purpose. Keep it updated and synced!
- `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `backend/Cargo.toml` have versions in there.
- Version in `package.json` is considered as root version. Others are synced with it.
