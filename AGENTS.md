# Elio Project Rules

## Project

Nightingale it is an app that helps clinics and specialists manage medical appointments, communicate with their patients, and reduce no-shows.

## Source of truth

The [Master Plan](./docs/master-plan.md) is the project source of truth.

- Read the relevant parts before plans, design choices, or product code changes.
- Keep its main boundaries: user-owned files, Markdown as readable text, a document model between text and the view, rebuildable indexes, and local use by default.
- Keep concepts separate when the spec does: folders and notebooks, Calendar and Agenda, Canvas and FreeLayout, Git history and undo, dictation and editing.
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
- [SQLite FTS5 documentation](https://www.sqlite.org/fts5.html)
- [Git documentation](https://git-scm.com/doc)

Agent workflow:

- [Codex guide for `AGENTS.md`](https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide#using-agentsmd)

Use the specification to select the right tool. Check current official documentation before using an API whose details can change.
