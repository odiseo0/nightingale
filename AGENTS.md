# Nightingale Project Rules

## Project

Nightingale helps doctors and specialists manage appointments, contact patients, and reduce missed visits. The first version serves independent practices and their assistants. Support for clinics with many doctors or sites comes later.

## Source of truth

The [Master Plan](./docs/master-plan.md) is the project source of truth.

- Read the relevant parts before plans, design choices, or product code changes.
- Run desktop use cases in Rust and save local data in SQLite. Local work must not need HTTP. Rust sends complete operations to Nightingale API over HTTPS and receives approved changes. Do not give the desktop SQL access or PostgreSQL credentials. Do not share database files between devices.
- Keep domain rules free of framework and storage code. React calls specific Rust commands through Tauri; React does not run SQL or hold cloud secrets.
- Keep appointment status separate from sync status. A local booking stays pending until Nightingale API confirms its PostgreSQL transaction. The API must check access, versions and shared rules even if Rust has checked them. Save the effect, prior-operation result, audit record and change record in one transaction.
- The Patient Hub manages existing appointments only. It owns patient access, policy and requests. Its backend calls Nightingale API to read or change shared appointments; it has no direct access to the operational database. Desktop Rust uses the Hub API and keeps a local view of its results. A request to change a time does not reserve a new time.
- Keep the last valid local permissions while offline. Permission changes need an authorized online operation. Check permission versions and revocations on reconnect; a client timestamp alone cannot prove that a change was allowed.
- Keep three projects in separate repositories: Nightingale (desktop), Nightingale Patient Hub (web and its API), and Nightingale API (Python/FastAPI, sync and shared transactions). Nightingale API also owns reminder, n8n integration and data-processing modules and workers. Run external work after commit; do not wait for it in a sync transaction. n8n is an external tool. Do not add cloud servers or web apps to this repository.
- Keep clinical records, diagnoses, and medical results outside the first version.
- The current foundation covers desktop development on Windows, macOS, and Linux. Web services need later plans; their absence does not block this foundation.
- A direct user request can change the spec. If a request conflicts with it and does not clearly replace it, state the conflict before implementation.
- Do not edit the spec as a side effect of code work. Change it only when the user asks or approves a spec change.

## Project workflow

- For each plan, name the native use case, component owner, local data, offline path, sync contract, and any cloud exception. Use the [planning checks](tasks/README.md#architecture-checks-for-each-plan). Define shared contracts before tasks run in parallel. Do not make a cloud API a prerequisite for native domain work.
- For a feature, broad change or work that spans parts of the app, use `$planning-task-breakdown` before implementation.
- Skip that skill for a small change with clear scope, or when an approved plan already defines the task.
- Before any code change, use `$minimal-code` after tracing the real flow. Stop at the first valid option: no change, reuse, standard library, native feature, installed dependency, one clear line, or minimum new code.
- For framework or library API code, use `$docs-first-framework-code`. Detect installed versions, use the exact official documentation page, and cite it in the handoff.
- Planning may change only planning files. Do not change product code while making the plan.
- Follow the plan and task names in `tasks/README.md`. Store each plan under `tasks/<plan-id>/` as `plan.md` and `todo.md`, unless the user names an external tracker.
- Do not start implementation until the user reviews and approves the plan.
- During implementation, work on one approved task at a time unless the approved plan names independent parallel tasks. Run each task's checks and update its checklist. Keep user review at each checkpoint.
- Preserve user work. Never replace an incomplete plan for different work without a direct choice from the user.

Available project skills:

- `$planning-task-breakdown`: Write a small, ordered, testable implementation plan and task list before broad work.
- `$docs-first-framework-code`: Verify framework and library code against version-matched official documentation and cite each non-trivial pattern.
- `$minimal-code`: Find the smallest correct change, fix root causes, avoid unneeded code, and keep comments and spacing useful.
- `$code-review`: Review code

## Documentation

Project:

- [Master Plan](./docs/master-plan.md)
- [Three projects and sync API](docs/decisions/transactional-sync-three-projects.md)
- [Nightingale API specification](docs/specifications/nightingale-api.md)
- [Nightingale Patient Hub specification](docs/specifications/nightingale-patient-hub.md)
- [Plan and task naming rules](tasks/README.md)

Core tools:

- [Rust documentation](https://doc.rust-lang.org/)
- [Tauri 2 documentation](https://v2.tauri.app/)
- [React documentation](https://react.dev/reference/react)
- [TypeScript documentation](https://www.typescriptlang.org/docs/)
- [Vite guide](https://vite.dev/guide/)
- [SQLite documentation](https://www.sqlite.org/docs.html)
- [PostgreSQL documentation](https://www.postgresql.org/docs/)
- [Git documentation](https://git-scm.com/doc)

Agent workflow:

- [Codex guide for `AGENTS.md`](https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide#using-agentsmd)

Use the specification to select the right tool. Check current official documentation before using an API whose details can change.
