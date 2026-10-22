# Laws of UX — Pre-Ship Checklist

Focused 12-point check before shipping frontend code. Report evidence and code fixes.

Use `laws-of-ux.md` as the main source when a check conflicts with a law.
Miller's Law does not set a fixed limit on visible options. Check the user's
task and what they must remember, rather than counting items alone.

## Process

### Step 1: Read the Component

Read the target file(s). If not specified, check the most recently modified frontend file.

### Step 2: Run the 12-Point Check

For each relevant item, read the actual code. Check the running UI when the
result depends on layout or timing. Use PASS or FAIL when the evidence supports
it. Use N/A when a check does not apply, or NOT CHECKED when evidence is missing.
Replace the result placeholders below with those labels. Report only the rows
that apply to the component.

```markdown
# Pre-Ship UX Check: [Component Name]

| # | Check | Law | Result | Fix |
|---|-------|-----|--------|-----|
| 1 | **Primary action is the most prominent element** — largest CTA, high contrast, easy to reach | Fitts's, Von Restorff | ✅/❌ | [one-line fix if ❌] |
| 2 | **Choices are clear and grouped by task** — reduce needless decisions; keep needed options easy to find and compare; do not impose a fixed item limit | Hick's, Miller's, Choice Overload | ✅/❌ | [fix] |
| 3 | **Related elements are visually grouped** — tight spacing within groups, generous between | Proximity, Common Region | ✅/❌ | [fix] |
| 4 | **Feedback within 400ms** — loading states, optimistic updates, skeleton loaders exist | Doherty Threshold | ✅/❌ | [fix] |
| 5 | **Follows platform conventions** — nav placement, form patterns, table behavior match expectations | Jakob's Law | ✅/❌ | [fix] |
| 6 | **Interactive elements have hover/focus states** — transitions on buttons, links, inputs | Aesthetic-Usability, Flow | ✅/❌ | [fix] |
| 7 | **Progress visible for multi-step flows** — stepper, progress bar, "step X of Y" | Goal-Gradient, Zeigarnik | ✅/❌ | [fix] |
| 8 | **Touch targets ≥ 44x44px** — buttons, links, toggles, checkboxes on mobile | Fitts's Law | ✅/❌ | [fix] |
| 9 | **Empty states are handled** — not blank screens, has illustration/message/CTA | Peak-End Rule | ✅/❌ | [fix] |
| 10 | **Information is chunked** — sections have headers, lists are grouped, long forms are divided | Chunking, Cognitive Load | ✅/❌ | [fix] |
| 11 | **Visual hierarchy is clear** — title > subtitle > body > caption, varying sizes and weights | Selective Attention, Prägnanz | ✅/❌ | [fix] |
| 12 | **Complexity is hidden from user** — smart defaults, advanced options tucked away | Tesler's Law, Occam's | ✅/❌ | [fix] |
```

### Step 3: Verdict

```
PASSED: [X]/[Y] verified checks
NOT CHECKED: [Z] applicable checks
N/A: [N] checks excluded
```

Count only PASS and FAIL in the verified total. Exclude N/A and NOT CHECKED.
Put failures that prevent task completion first, even when other checks pass.
Report missing checks and remaining issues. A score alone does not show that
the UI is ready for release.

### Step 4: Quick Fixes (if any FAIL)

For each failed check, provide a single code block with the minimum change needed:

````markdown
## Fixes

### Check #[N]: [Check name]
```[language]
// [file:line] — add/change this
[code fix]
```
````

## Rules

1. **Read the code** — don't guess, verify each check against actual source
2. **Use evidence** — use PASS or FAIL for verified checks; report missing evidence as NOT CHECKED
3. **Provide code fixes** — summarize each fix in the table, then give the complete code needed below it
4. **Skip N/A checks** — if a component has no multi-step flow, exclude check #7 from the pass count
5. **Keep the check focused** — state any limits; do not rush a check and claim it passed
