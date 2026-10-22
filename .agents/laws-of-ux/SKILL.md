---
name: laws-of-ux
description: Apply UX laws when building, changing, or reviewing UI components and user flows. Select laws for the current task, find concrete problems, and provide code fixes in the project's stack. Use for frontend UX work or a UX check before release, not backend-only changes.
---

# Laws of UX

Use these laws to help users complete their tasks with less effort. Apply them
to the current component and its role in the user flow.

## When to Use

- Build or change a form, navigation, dashboard, table, dialog, or other UI.
- Review UI code for problems that affect a user's task.
- Fix a specific UX issue, such as unclear choices or missing feedback.
- Run a focused UX check before release.

Use the skill during normal UI work when relevant. The user can also request
it by name, for example: "Use $laws-of-ux to review this form."

## Critical Rules

1. **Never list all 30 laws at once** - show only what is relevant to the current component.
2. **Always provide code** - if you cannot write a CSS/JSX fix, the advice is too vague. Use the equivalent code in the user's stack when it does not use JSX.
3. **Violations > Suggestions** - put confirmed problems before optional improvements.
4. **Respect the user's stack** - adapt patterns to their framework and styles, such as React, Vue, Tailwind, or CSS modules.
5. **Don't repeat yourself** - if you already applied a law in earlier components, skip it unless you find a new violation.

## Sources

Resolve these paths relative to this skill folder, not the project being reviewed.

- [laws-of-ux.md](references/laws-of-ux.md): main source for each law and its takeaways.
  Read the entries for the laws you plan to apply. This file takes priority
  when a supporting file conflicts with it.
- [ux-patterns.md](references/ux-patterns.md): use the component table to find laws to
  check first. Read code examples only when they help the current fix.
- [design-principles.md](references/design-principles.md): read when the issue involves
  too much mental effort, unclear content, or needless user tasks.
- [ux-checklist.md](references/ux-checklist.md): read for a requested check before release.
  Apply its checks to the current component and report only relevant results.

The examples show possible fixes. They do not set a required visual style,
framework, layout, or item count.

## How to Use

### 1. Establish the Task

Read the target code and the related styles, shared controls, and state code
needed to understand its behavior. Identify the user's goal, the component
type, the platform, and the project's framework and style system.

Keep work within the user's request. A review calls for findings and proposed
code; a build or fix request calls for edits. Reuse existing controls and styles.
For a new component, use the requirements and nearby UI as context. If needed
code is missing, request only that code. Label any sample based on assumptions.

### 2. Select Relevant Laws

Use the component table in `references/ux-patterns.md` as a starting point. For a component
with several roles, check the rows for those roles. The table is not a complete
list of possible issues; check another law when the observed behavior calls for it.

For each candidate law:

- Read its entry in `references/laws-of-ux.md`.
- Identify the code or behavior that affects the user's task.
- Keep the law only if it explains a concrete problem or a needed design choice.
- Combine overlapping laws under one finding when they support the same fix.

For example, a long form with unclear groups may need Chunking and Proximity.
A search that gives no feedback while waiting may need the Doherty Threshold.
Do not add unrelated laws to make the review longer.

Keep track of fixes and laws already covered in the current conversation.
Check new components for new violations, but do not repeat earlier explanations
or advice when no new issue exists.

### 3. Apply the Laws in Context

Miller's Law concerns memory. It does not set a maximum of seven visible options,
columns, metrics, or navigation links. Check what users must remember and group
related content. Keep values needed for comparison visible together.

Use Hick's Law and Choice Overload to reduce needless decisions when they hinder
the task. Do not hide required choices just to meet a count. Keep less visible
options easy to find when you use progressive disclosure.

Distinguish a law from an example. A short CSS transition does not prove that a
system gives feedback within 400 ms. A loading state in source code does not
prove its timing. Keep controls clear and usable when adapting any example.

### 4. Provide Concrete Code

Start with confirmed violations. Order them by their effect on task completion,
then give optional improvements if they add value.

For each finding, provide:

- The file and location, when code exists.
- The observed problem and its effect on the user.
- The relevant law, with a brief reason.
- The smallest complete code fix in the project's stack.

Show enough code to apply the fix, including required state, handlers, or styles.
Do not omit a needed fix because it exceeds a line limit. Avoid new dependencies
when the project already has a suitable control or a native element will work.

For build requests, apply the fixes and show the key changed code in the response.
For reviews, provide proposed code without editing unless the user asks for edits.
If no violation is found, say so and show a short existing code excerpt as evidence;
do not invent a problem to produce a fix.

### 5. Verify and Report

Use the project's existing checks for the changed code. Inspect the UI when
layout, focus, motion, or timing matters and a suitable preview is available.
State which checks you ran and which behavior you could not verify.

In a checklist review, mark checks that do not apply as N/A and leave them out
of the pass count. Mark checks without enough evidence as NOT CHECKED. Do not
count either as a pass, and do not claim measured behavior from code alone.
A checklist score is a limited review result, not proof that the UI is ready
for release.

Keep the final response focused on violations and code fixes, followed by useful
optional changes and the checks performed. For a new component, briefly explain
the laws behind the key design choices without listing every law considered.
