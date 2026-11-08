# SaaS interface design rules

## Core idea

Good code does not make a SaaS product look professional. Clear choices, strong structure, and consistent details do.

Users care about the result. Guide them toward it and remove anything that gets in the way.

## 1. Start with the user’s task

1. **Give each page one job.** Ask why the user opened it and remove content that does not support that purpose.
2. **Give each screen one main element.** Use size, weight, color, and space to show where the user should look first.
3. **Start with purpose, not components.** Choose cards, filters, forms, and menus only after you understand the task.
4. **Add features only for real needs.** Do not add something because there is free space or because it might be useful.
5. **Remove before you add.** A simpler screen is often clearer than a more complete one.

## 2. Follow one design system

6. **Use `DESIGN.md` as the source of truth.** Define the type scale, spacing, colors, borders, corners, icons, and component states.
7. **Use one name for each action.** Do not use “Delete,” “Remove,” and “Trash” for the same result.
8. **Keep text styles consistent.** Use Anek Latin for titles and headings, and Aileron for general interface and reading text. Define clear styles for body text, labels, and supporting details.
9. **Use strong type contrast.** Important text must stand apart from supporting text, but small text must remain easy to read.
10. **Use technical text styles with care.** Keep IDs, dates, metrics, units, and code in Aileron; use tabular numerals when alignment matters instead of introducing another font family.
11. **Keep corner shapes consistent.** Use a small set of corner sizes and give each one a purpose.
12. **Use Phosphor Icons as the one icon set.** Keep icon size, weight, and style consistent. Do not mix icon libraries or use emojis as interface icons.

## 3. Keep the interface quiet

13. **Give color a purpose.** Use it for status, data, warnings, progress, and main actions.
14. **Use a restrained palette.** Start with quiet base colors and a small number of accents.
15. **Use thin borders and few shadows.** Add depth only when it explains focus or an active layer.
16. **Do not make everything a card.** Use spacing and dividers when a container adds no meaning.
17. **Reduce repeated content.** Keep only the details needed to scan and compare items.
18. **Hide secondary actions.** Put rare actions in an ellipsis menu, but keep common actions visible.
19. **Emphasize the main value.** Give the most useful number or state a clear and stable position.
20. **Use the correct container.** Match dialogs, panels, and pages to the amount of work they contain.
21. **Remove decoration without a job.** Each element must explain, support an action, or confirm a result.

## 4. Design for real content

22. **Test with difficult data.** Use long names, missing values, large numbers, and many rows.
23. **Define how long text behaves.** Decide when it wraps, shortens, or expands.
24. **Check every visual state.** Icons and text must remain clear when selected, disabled, or placed on color.
25. **Design empty, loading, error, disabled, and success states.** These states are part of the product.
26. **Do not replace clear words with unclear icons.** Use text labels for actions that users may not know.

## 5. Show progress and results

27. **Respond to every action.** Give immediate proof that the product received the user’s input.
28. **Show progress for slow work.** Use a spinner, skeleton, progress bar, or useful status message.
29. **Show clear completion.** The user should not need to refresh the page to check if something worked.
30. **Mark useful milestones.** Celebrate meaningful progress, not every small action.
31. **Show the value created.** Display useful results such as time saved or work completed.

## 6. Add safe friction

32. **Confirm serious actions.** Add friction before destructive, costly, or final actions.
33. **Explain the result.** State what will change, what will disappear, and whether it can be restored.
34. **Make dangerous actions harder.** For serious cases, require a name or confirmation word.
35. **Offer Undo when possible.** Show what happened and give the user time to reverse it.

## 7. Use motion with purpose

36. **Give every animation a job.** Motion should explain loading, progress, movement, or completion.
37. **Avoid decorative effects.** Remove needless parallax, scroll effects, entrance effects, and constant fading.
38. **Keep transitions precise.** Movement should help the user understand what changed.
39. **Use “Load more” when users need control.** Give them a clear stopping point instead of endless scrolling.

## 8. Keep landing pages focused

40. **Make the offer clear at once.** Explain the product, user, result, and next action.
41. **Avoid repeated left-and-right sections.** Change the layout to match the content.
42. **Show the real product.** Replace stock images with focused product views.
43. **Keep calls to action consistent.** The same label should lead to the same place.
44. **Describe outcomes, not only features.** Explain how the product helps the customer.
45. **Include only useful sections.** Use the promise, proof, process, price, questions, and final action as a base.

## 9. Always apply UX laws

46. **Use `.agents/laws-of-ux/SKILL.md` for every UI/UX design, build, change, or review.** Establish the user's task, select only the relevant laws, and tie each decision to concrete behavior or code. Put confirmed violations before optional suggestions and verify the running UI when layout, focus, motion, or timing matters. Do not list all laws indiscriminately or treat their examples as universal requirements.

## Visual direction

Prefer strong type contrast, quiet colors, clear borders, real product views, varied layouts, and direct copy.

Avoid purple gradients, glossy 3D shapes, generic stock photos, rounded containers everywhere, forced playful language, emoji icons, repeated icon grids, equal space for unequal content, and many accent colors.

These are default choices, not absolute rules. Break them only when the product has a clear reason.
