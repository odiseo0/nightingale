# Evidencia C03 · Base visual y lecturas locales

C02 aprobado por el usuario. Se conserva su indicación: mayor fidelidad a Linear y Attio según DESIGN.md. C03 pendiente de revisión; no comenzar T08 antes de aceptación expresa.

## Cambios

### T05 · Fuentes

App.css carga Montserrat para títulos y Mulish para lectura/controles, con pesos variables reales 400–600 y fallback de sistema. Reutiliza los archivos de la muestra aceptada y conserva origen, huellas y licencias completas en src/assets/fonts/LICENSE.md. Las dos fuentes antiguas y la dependencia Anek se conservan hasta T08, pero App.css ya no las carga.

No se rediseñan componentes en este bloque. Se conservan paleta, superficies, radios 6/8/12, foco, movimiento reducido, título de 24 px, cuerpo de 16 px y controles de 14 px/36 px de la base existente conforme a DESIGN.md. Los cambios de estructura y composición pertenecen a tareas posteriores. No se añade un estilo distinto de Linear/Attio ni una nueva paleta.

### T06 · Rol local

AccountScreen/AccountScreenDto y su cliente TypeScript añaden viewerRole: owner | assistant. El caso de uso copia AccessRole del acceso local seleccionado y autorizado, después de exigir acceso activo; no lo deduce de canManage. No añade HTTP ni cambia permisos.

La prueba de la lectura real read_view usa SQLite en memoria y cubre dueño sin gestión, dueño con gestión y asistente con acciones de gestión presentes. Verifica serialización, preferencias aisladas por cuenta/usuario/dispositivo, ausencia de escrituras en la lectura y rechazo de acceso revocado, inactivo o sin registro. Los tres casos no necesitan un servidor.

### T07 · Metadatos de revisión

ReviewSummary añade action, entityType, entityKey y createdAtMillis desde columnas ya guardadas de sync_outbox. Reutiliza json_column y la misma consulta parametrizada. Conserva filtros por cuenta/usuario/dispositivo/versión de permisos, estados, orden y límite de 100. No añade migración ni cambia intención, identidad, payload o resultado.

entityKey conserva su JSON, incluida clave compuesta; TypeScript lo recibe como unknown hasta que un consumidor compruebe su forma. CoreStatus reutiliza el tipo en sync.ts. No se muestran todavía estos datos como contenido del producto ni se cambia la clasificación de Por revisar (T27).

La prueba de lectura cubre resultado vacío, claves simples/compuestas, fecha original, código, estado y orden, exclusión de otras cuentas/usuarios/dispositivos y exclusión del unknown normal. Se conserva la selección previa de casos que necesitan revisión. Comprueba ausencia de escrituras y de payload, resultados y otros campos privados. La prueba propietaria de DesktopView comprueba serialización sin datos privados.

## Comprobaciones

Los resultados siguientes corresponden a los comandos terminados. Los archivos check-t06-*.log y check-t07-*.log conservan salida de comandos en datos de prueba.

- T05 pnpm quality: PASS. Build, formato, Clippy; 375 unitarias, 5 persistencia y 1 doc-test pasan, 9 ignoradas. Los WOFF2 emitidos por Vite coinciden byte a byte con los archivos locales.
- T06 cargo test --workspace --locked account: PASS, 30 pruebas. Primer intento en sandbox: 7 fallos de transporte local compatibles con las restricciones del sandbox; repetición autorizada sin sandbox: 30 correctas sin cambios de código.
- T06 pnpm quality: PASS. Build, formato, Clippy; 376 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas.
- T07 cargo test --workspace --locked sync: PASS, 80 correctas y 3 ignoradas.
- T07 pnpm quality final: PASS. Build, formato y Clippy; 377 unitarias, 5 persistencia y 1 doc-test correctas (383 en total), 9 ignoradas.
- git diff --check: PASS. Sin cambios de contratos exportados, migraciones, manifiestos ni estados de planes anteriores.
- Inspección visual de app, glifos renderizados, Agenda/editor/tabla, zoom real 200 %, foco y movimiento reducido en ejecución: **NOT CHECKED**. Los estilos y recursos presentes no acreditan su render.
- Lectura desde la app con cuentas/perfiles reales y revisiones reales de prueba: **NOT CHECKED**. Las pruebas automatizadas verifican los casos de uso y serialización, no una sesión nativa interactiva.
- Corte real de red, API real/OIDC, macOS/Linux, lector y validación humana: **NOT CHECKED**.
- Revisión del usuario de C03: **NOT CHECKED**, pendiente.

Vite avisa de un chunk JavaScript de más de 500 kB. No se cambia la división de módulos en estas tareas. El compilador del primer intento bajo sandbox avisó que copiaría archivos por no poder crear enlaces duros en la caché; no se modifica código para ello. Las 9 pruebas ignoradas no se cuentan como ejecutadas.

## Skills, versiones y fuentes

planning-task-breakdown mantiene el plan, las tareas y sus checkpoints. minimal-code: reutilizar fuentes, enum AccessRole, DTO y consulta existentes; sin dependencia, abstracción, migración ni cambio del contrato HTTP. Laws of UX mantiene lectura clara y grupos/controles ya definidos, sin una nueva composición en estas tareas.

Versiones del lockfile: Vite 8.3.4, React 19.3.1, Router 7.18.5, Tauri API 2.12.4, TypeScript 6.0.3, rusqlite 0.40.2 y serde 1.0.229. Patrones de docs-first-framework-code:

- [Vite: assets y url() en CSS](https://vite.dev/guide/assets.html#importing-asset-as-url): empaquetado de las fuentes locales, confirmado en el build 8.3.4.
- [MDN: @font-face](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@font-face#descriptors): rango real de pesos y fuentes locales.
- [Serde: rename_all](https://serde.rs/container-attrs.html#rename_all): campo viewerRole y valores owner/assistant con el enum existente.
- [rusqlite 0.40.2: query_map](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Statement.html#method.query_map): ampliar columnas de la consulta existente sin alterar su filtro.

## Revisión de C03

Revisar el diff de App.css y los archivos de fuentes; viewerRole en aplicación/DTO y TypeScript; campos de ReviewSummary y sus pruebas. Los logs documentan comprobaciones automatizadas. Se requiere revisión expresa antes de T08. Ningún plan funcional previo cambia de estado ni se cierra.
