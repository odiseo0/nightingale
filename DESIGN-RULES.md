# SaaS interface design rules

## Revisión aprobada del 2026-11-09

El plan [P-20261109-22](tasks/P-20261109-22-interface-refresh/plan.md) fija estas reglas de presentación. No cambia dominio ni contratos.

- Montserrat para títulos y encabezados; Mulish para lectura, navegación, tablas, formularios y controles. Pesos reales 400/500/600, archivos locales con origen y licencia.
- La sincronización normal es invisible: sin columnas, etiquetas, contadores, bordes discontinuos, botones ni avisos de proceso. Los reintentos automáticos y la falta de red que permite trabajo local no interrumpen.
- Los problemas que exigen una decisión aparecen junto al dato y en «Por revisar». Su enlace solo aparece si hay intervención pendiente. Diagnóstico conserva los datos técnicos fuera del uso diario.
- «Cita guardada» o «Cambios guardados» confirma persistencia local. No promete reserva compartida, mensaje enviado ni horario liberado. El resultado desconocido conserva la misma intención e impide duplicarla.
- Las direcciones de lectura terminan en ciudad y estado, sin país ni separadores vacíos. País almacenado/editable y país emisor de documentos se conservan.
- Ocultar columna y filtro Profesional solo para el dueño con un único profesional disponible. Asistentes y cuentas con varios profesionales lo conservan. No inferir rol por canManage ni quitar professionalId.
- Conservar Servicio y su dato actual; no añadir motivo de visita.
- Tomar de Linear, Attio y Twenty navegación, barras compactas, tablas, filtros, paneles y formularios. Conservar azul propio, Phosphor y alcance de Nightingale.
- Semana parte de 96 px/hora y al menos 160 px/día, con desplazamiento propio, horas/cabeceras identificables y eventos sin taparse. La muestra se revisa antes de llevar las medidas al producto.


## Reglas tipográficas del 2026-11-12

- **Longitud de lectura:** entre 45 y 75 caracteres por línea completa de texto explicativo, incluidos espacios, cuando el ancho lo permita. Usar unos 60ch como punto de partida y revisar el texto real. No imponer un ancho mínimo en móvil, tarjetas o modales estrechos; nombres, fechas, etiquetas, celdas y últimas líneas pueden ser más cortos.
- **Tamaños de 4 en 4:** 12, 16, 20, 24, 28, 32px y siguientes múltiplos de 4. Usar los roles de `DESIGN.md`, sin tamaños intermedios ni texto inferior a 12px. Conservar el aumento de fuente y el zoom que elija el usuario.
- **Interlineado por función:** títulos compactos (1.00–1.25); lectura a partir de 1.50; texto de apoyo de 12px con más aire (aprox. 1.67). Revisar títulos de varias líneas y conservar la relación visual entre etiquetas, datos y grupos.

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
8. **Keep text styles consistent.** Use Montserrat for titles and headings, and Mulish for general interface and reading text. Define clear styles for body text, labels, and supporting details.
9. **Use strong type contrast.** Important text must stand apart from supporting text, but small text must remain easy to read.
10. **Use technical text styles with care.** Keep IDs, dates, metrics, units, and code in Mulish; use tabular numerals when alignment matters instead of introducing another font family.
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
28. **Show progress for slow user work.** Use a spinner, skeleton, progress bar, or useful status message. Keep normal background sync silent.
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
