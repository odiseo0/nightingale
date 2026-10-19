# Nightingale Project Rules

## Project

Nightingale helps doctors and specialists manage appointments, contact patients, and reduce missed visits. The first version serves independent practices and their assistants. Support for clinics with many doctors or sites comes later.

## Source of truth

The [Master Plan](./docs/master-plan.md) is the project source of truth.

- Read the relevant parts before plans, design choices, or product code changes.
- Use SQLite for local work on each device. PostgreSQL holds the final shared state. Send changes through the API; do not share or replicate database files.
- Keep domain rules free of framework and storage code. React calls specific Rust commands through Tauri; React does not run SQL or hold cloud secrets.
- Keep appointment status separate from sync status. A local booking stays provisional until the server accepts it.
- The Patient Hub manages existing appointments only. A request to change a time does not reserve a new time.
- Keep clinical records, diagnoses, and medical results outside the first version.
- The current foundation covers desktop development on Windows, macOS, and Linux. Web services need later plans; their absence does not block this foundation.
- A direct user request can change the spec. If a request conflicts with it and does not clearly replace it, state the conflict before implementation.
- Do not edit the spec as a side effect of code work. Change it only when the user asks or approves a spec change.

## Project workflow

- For a feature, broad change or work that spans parts of the app, use `$planning-task-breakdown` before implementation.
- Skip that skill for a small change with clear scope, or when an approved plan already defines the task.
- Before any code change, use `$minimal-code` after tracing the real flow. Stop at the first valid option: no change, reuse, standard library, native feature, installed dependency, one clear line, or minimum new code.
- For framework or library API code, use `$docs-first-framework-code`. Detect installed versions, use the exact official documentation page, and cite it in the handoff.
- Planning may change only planning files. Do not change product code while making the plan.
- Follow the plan and task names in `tasks/README.md`. Store each plan under `tasks/<plan-id>/` as `plan.md` and `todo.md`, unless the user names an external tracker.
- Do not start implementation until the user reviews and approves the plan.
- During implementation, work on one approved task at a time. Run its checks and update its checklist before starting the next task.
- Preserve user work. Never replace an incomplete plan for different work without a direct choice from the user.

Available project skills:

- `$planning-task-breakdown`: Write a small, ordered, testable implementation plan and task list before broad work.
- `$docs-first-framework-code`: Verify framework and library code against version-matched official documentation and cite each non-trivial pattern.
- `$minimal-code`: Find the smallest correct change, fix root causes, avoid unneeded code, and keep comments and spacing useful.
- `$code-review`: Review code

## Documentation

Project:

- [Master Plan](./docs/master-plan.md)
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
