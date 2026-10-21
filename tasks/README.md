# Plan and Task Names

Use one stable ID for each body of planned work.

## Plan ID

Format:

```text
P-YYYYMMDD-NN-short-slug
```

Example:

```text
P-20261006-01-vault-foundation
```

- `P` means plan.
- `YYYYMMDD` is the local creation date.
- `NN` is the next two-digit plan number for that date, starting at `01`.
- `short-slug` is two to five lowercase words that name the outcome.

Scan `tasks/` before assigning `NN`. Do not reuse or rename a plan ID after creation.

## Plan files

Each plan has its own folder:

```text
tasks/
└── P-20261006-01-vault-foundation/
    ├── plan.md
    └── todo.md
```

Keep completed and superseded plan folders. They are project history.

If the user selects an external tracker, keep `plan.md` in the plan folder and put tracker links in it. Do not create a duplicate `todo.md`.

## Task and checkpoint IDs

Use task IDs in order:

```text
T01
T02
T03
```

Use checkpoint IDs in order:

```text
C01
C02
```

Within a plan, short references such as `T02` are enough. Across plans, use the full reference:

```text
P-20261006-01-vault-foundation/T02
```

Do not renumber tasks or checkpoints after the user approves the plan. Append new IDs instead.

## Status values

Plan status:

```text
Draft
Approved
In progress
Blocked
Completed
Superseded
```

Task status:

```text
Todo
In progress
Blocked
Done
```

Only the user can move a plan from `Draft` to `Approved`. Keep at most one task `In progress` unless the approved plan calls for parallel work.

When a plan is superseded, keep it and link to the plan that replaces it.

Do not reopen or rewrite a completed or superseded plan. Create a new plan ID and link the related older plan.

## Architecture checks for each plan

Read the current [Master Plan](../docs/master-plan.md) and the [three-project decision](../docs/decisions/transactional-sync-three-projects.md). Closed plans record past work; they do not override the current architecture.

Each new plan must state:

- The component owner, native Rust use case, local SQLite data, and public interface.
- What works without a network, which data must already be local, and how saved changes survive a restart.
- How pending operations, conflicts, permissions and versions enter the HTTPS sync contract owned by Nightingale API. The desktop must not have SQL access or PostgreSQL credentials.
- Each cloud owner and dependency. Patient Hub owns patient access and policy; its backend calls Nightingale API to read or change appointments. Rust consumes the Hub API without copying its rules. Local use cases must not need an HTTP service. Nightingale API owns shared acceptance and its own worker modules.
- The focused checks for no connection, a failed cloud service, and recovery when these affect the change. State which remote checks are outside this plan.
- Which contracts and migrations must be fixed before independent tasks can run in parallel. Name the shared files and integration owner.

Keep one owner for each contract source across the three repositories. Nightingale API owns sync contracts and operational database migrations. Patient Hub owns its public contract and private data. Keep cloud servers and web apps out of desktop implementation plans; their initial specifications may live here until an explicit move to their own repositories. External work runs after commit, outside sync transactions.

Offline work uses the last valid permissions. Permission changes need a connection. Plans for identity or sync must define how permission versions and trusted effective dates govern pending operations; a client date alone is not proof of access.

For a base that enables later work, give each task a reason, the components it enables, and its prerequisites. Do not create empty modules only to match a directory tree.
