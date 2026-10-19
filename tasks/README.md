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
