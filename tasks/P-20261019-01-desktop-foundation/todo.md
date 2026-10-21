# Task List: P-20261019-01-desktop-foundation

**Plan:** [plan.md](plan.md)
**Updated:** 2026-10-21

El usuario autorizó comenzar el plan el 2026-10-19. Antes de cada cambio de código, aplicar `minimal-code`; antes de usar APIs de bibliotecas, aplicar `docs-first-framework-code`. Una tarea activa cada vez. No interpretar los comandos futuros como comprobaciones ya aprobadas.

## T01: Corregir las instrucciones contradictorias

**Status:** Done

**Description:** Alinear las referencias de trabajo con las decisiones ya descritas para Nightingale. Retirar reglas de Elio, corregir la creación de citas desde Patient Hub en la SRS y registrar las dudas de dominio pendientes para sus rutas futuras. La corrección del alcance funcional requiere la aprobación de este plan; no añadir reglas nuevas por deducción.

**Acceptance criteria:**
- [x] `AGENTS.md` describe Nightingale, su persistencia operativa y el flujo de tareas; no impone archivos Markdown como datos del producto.
- [x] La SRS coincide con PMD/DAT sobre segmento inicial y Patient Hub; distingue alcance incluido de trabajo implementado en sus casillas.
- [x] Las dudas sobre `reschedule_requested` y el diagrama de recordatorios quedan registradas sin inventar una solución.

**Verification:**
- [x] Prueba enfocada: buscar las reglas heredadas con `rg -n 'Elio|notebooks|FreeLayout|user-owned files' AGENTS.md` y revisar SRS §§7, 9, 16 contra PMD §§4, 17.
- [x] Revisión amplia: `git diff --check`; comparar las referencias entre los tres documentos.
- [x] Revisión manual: una persona identifica sin contradicciones qué producto se construye y qué queda fuera de esta base.

**Evidence (2026-10-19):** búsqueda de reglas heredadas sin coincidencias; SRS 0.3 y referencias PMD/DAT coherentes. Revisión de las reglas de Patient Hub, segmento inicial y alcance sin casillas de progreso. Comprobación de enlaces locales: sin rutas rotas. `git diff --check`: pasa. Se anotaron las dos dudas futuras en DAT §§11 y 12.7 sin cambiar sus transiciones o diagramas. La revisión del usuario sigue pendiente en C01; no se ha modificado código de producto.

**Dependencies:** None
**Files likely touched:** `AGENTS.md`, `docs/srs.md`, `docs/technical-architecture.md`, `docs/master-plan.md`.
**Estimated scope:** M

## T02: Acordar el entorno de desarrollo

**Status:** Done

**Description:** Registrar una decisión de la base: mapa de rutas, versiones de herramientas, sistemas y arquitecturas cubiertos, requisitos nativos y comandos raíz. Verificar compatibilidad con los manifiestos antes de recomendar versiones. Mantener fuera las decisiones del servidor.

**Acceptance criteria:**
- [x] Un ADR explica el mapa `src/`/`src-tauri/`, sus límites y la incorporación posterior de aplicaciones web sin crear carpetas vacías.
- [x] El ADR fija la matriz inicial de Windows/macOS/Linux y versiones compatibles de Node, pnpm y Rust con fuentes oficiales.
- [x] El PMD enlaza el ADR y reconoce la decisión de las tres plataformas sin retirar servicios del alcance futuro del producto.

**Verification:**
- [x] Prueba enfocada: cotejar versiones propuestas con `package.json`, los manifiestos Rust y las páginas oficiales de requisitos.
- [x] Revisión amplia: `git diff --check`; comprobar enlaces relativos del ADR y PMD.
- [x] Revisión manual: el usuario revisa sistemas, arquitecturas, versiones y mapa de rutas en C01.

**Evidence (2026-10-19):** ADR creado con estado Propuesta. Las versiones locales Node 24.18.0, pnpm 11.17.0 y Rust/Cargo 1.99.1 coinciden con releases oficiales consultadas. Los manifiestos oficiales de Vite 8.0.16 y plugin React 6.0.2 admiten Node 24; plugin React admite Vite 8. Las dependencias completas aún no se han instalado ni verificado. La matriz propuesta cubre Windows 11 x64, macOS 15 ARM/Intel y Ubuntu 24.04 x64, con etiquetas CI disponibles según GitHub. PMD y DAT ya distinguen las tres plataformas confirmadas de las versiones concretas pendientes. Enlaces locales y `git diff --check` pasan. No hay remoto Git configurado: la ejecución de CI seguirá pendiente hasta disponer de uno. No se ha cambiado código, instalado paquetes ni iniciado T03. Falta la revisión del usuario en C01.

**Dependencies:** T01
**Files likely touched:** `docs/decisions/desktop-foundation.md`, `docs/master-plan.md`, `docs/technical-architecture.md` (tabla de plataformas para evitar una referencia contradictoria).
**Estimated scope:** S

## C01: Después de T01–T02

- [x] Reglas y decisiones revisadas en sus fuentes; solo han cambiado documentos. La aceptación del usuario queda en la última casilla.
- [x] Las elecciones de T02 permiten ejecutar las tareas siguientes sin suponer plataformas o versiones.
- [x] Revisión del usuario completa.

**Acceptance (2026-10-19):** el usuario respondió «Puedes continuar, por favor» a la petición de aceptar C01. Se aceptan el ADR, las herramientas, las rutas actuales y la matriz de cuatro hosts. Esta aceptación no incluye los checkpoints posteriores.

## T03: Reparar el workspace Rust

**Status:** Done

**Description:** Sustituir el paquete raíz inválido por un workspace virtual del escritorio, situar el perfil de compilación en raíz y generar un lockfile que represente Tauri. Resolver el ejemplo aislado `crates/main.rs` sin crear otro ejecutable innecesario. Fijar la herramienta acordada en T02.

**Acceptance criteria:**
- [x] Cargo descubre `src-tauri` desde raíz y desde su manifiesto; no queda un paquete raíz sin target.
- [x] El perfil de compilación y el lockfile pertenecen al workspace; una resolución bloqueada no requiere cambios de lockfile.
- [x] El ejemplo aislado ya no confunde la entrada del programa y el toolchain coincide con T02.

**Verification:**
- [x] Prueba enfocada: `cargo metadata --no-deps --format-version 1` y `cargo metadata --manifest-path src-tauri/Cargo.toml --no-deps --format-version 1`.
- [x] Compilación/revisión amplia: `cargo check --workspace --locked`; registrar requisitos nativos ausentes si los hay.
- [x] Revisión manual: las dos entradas de Cargo identifican el mismo workspace y la biblioteca del escritorio.

**Evidence (2026-10-19):** workspace virtual con miembro `src-tauri`, resolver 3 y perfil release en raíz. Se retiró el ejemplo no usado `crates/main.rs`. Rust 1.99.1 instalado y fijado con rustfmt/clippy. Ambos comandos de metadatos pasan (también con `--locked --offline`) y muestran el mismo workspace, biblioteca y binario. `cargo generate-lockfile` resuelve 449 paquetes: Tauri 2.12.4, tauri-build 2.7.3 y opener 2.7.2. `cargo check --workspace --locked` pasa en Windows en 1m31s; el hash SHA256 del lockfile permanece `64E87042794B4AA571A9CB7DB355DD21D42505BCB8C92F463A31E016D4A9968A`. `cargo fmt --all -- --check` y `git diff --check` pasan. Los errores iniciales de permisos rustup y DNS de crates.io desaparecieron al repetir con el acceso permitido; no se cambiaron dependencias para ocultarlos. Fuentes: [Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html#virtual-workspace) y [rustup toolchain file](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file).

**Dependencies:** T02, C01
**Files likely touched:** `Cargo.toml`, `Cargo.lock`, `src-tauri/Cargo.toml`, `rust-toolchain.toml`, `crates/main.rs`.
**Estimated scope:** M

## T04: Fijar la instalación JavaScript

**Status:** Done

**Description:** Declarar las herramientas acordadas y generar el lockfile JavaScript con pnpm. Mantener los comandos existentes y comprobar que funcionan con el workspace Rust reparado. No actualizar versiones principales sin explicar una incompatibilidad real.

**Acceptance criteria:**
- [x] El repositorio declara Node/pnpm y contiene un lockfile coherente con `package.json`.
- [x] Una instalación con lockfile fijo no modifica manifiestos ni archivos de bloqueo.
- [x] La interfaz compila y `pnpm tauri dev` abre la plantilla y permite ejecutar su saludo en el host disponible.

**Verification:**
- [x] Prueba enfocada: `pnpm install --frozen-lockfile` tras generar el lockfile; comprobar `git diff` de manifiestos/lockfile antes y después.
- [x] Compilación/revisión amplia: `pnpm build` y `cargo check --workspace --locked`.
- [x] Revisión manual: `pnpm tauri dev`; abrir y cerrar la ventana tras comprobar la respuesta Rust.

**Comando nativo para T07/T09:** `pnpm tauri build --debug --no-bundle`. Las opciones se comprobaron mediante `pnpm tauri build --help` de la CLI 2.12.3. No genera instaladores ni publica releases. En T04 se compiló y ejecutó el binario de desarrollo; el comando de build sin empaquetado se ejecutará en T07 para comprobar también la CSP con recursos compilados.

**Evidence (2026-10-19):** Node 24.18.0 declarado en `.node-version` y `engines`; pnpm 11.17.0 en `packageManager`/`engines`. Se generó `pnpm-lock.yaml`; resuelve React/React DOM 19.3.1, Vite 8.3.4, plugin React 6.1.2, TypeScript 6.0.3, API Tauri 2.12.4, CLI 2.12.3 y opener 2.7.2, dentro de los rangos previos. `pnpm install --frozen-lockfile` pasa sin modificar manifiesto o lockfile (SHA256 pnpm-lock: `E653D92273A3B09E2B14493E058943331511329E4CF52353253111DCB3059553`; package.json: `D3DA36C7727EABC842766BB5DBE3B4D2D5CE655CBB766BBF01ABE24F9A8D53A3`). `pnpm build` pasa. El arranque mostró EBUSY porque Vite vigilaba `target/`; se corrigió su filtro nativo y la recompilación web pasó. `pnpm tauri dev --no-watch` compiló el binario Windows en 2m28s, abrió la ventana y respondió al botón Greet desde Rust. Se verificó con Computer Use la ventana correcta, se pulsó Greet sin cambiar la entrada existente y se comprobó el texto devuelto. La ventana se cerró y la sesión terminó con código 0. No se afirman pruebas de macOS/Linux. El intento de pasar flags Cargo a través del lanzador PowerShell consumió el separador `--`; la comprobación de ejecución usó el comando normal sin esos flags y el lockfile Rust mantuvo su hash. Fuentes: [pnpm manifest](https://pnpm.io/package_json#engines), [instalación fija](https://pnpm.io/cli/install#--frozen-lockfile), [Vite watcher](https://vite.dev/config/server-options.html#server-watch), [CLI Tauri 2](https://v2.tauri.app/reference/cli/#build).

**Dependencies:** T03
**Files likely touched:** `package.json`, `pnpm-lock.yaml`, `.node-version`, `vite.config.ts` (ignorar `target/` tras mover la salida Cargo al workspace).
**Estimated scope:** M

## C02: Después de T03–T04

- [x] Cargo carga el escritorio; la instalación fija y la compilación web pasan.
- [x] La plantilla abre en el host disponible; constan los otros hosts aún pendientes.
- [x] Revisión del usuario completa.

**Acceptance (2026-10-19):** el usuario respondió «Lo veo todo bien, podemos continuar». Se acepta C02 y se autoriza el siguiente grupo T05–T07; C03 conserva su revisión pendiente.

## T05: Integrar Tailwind para estilos básicos

**Status:** Done

**Description:** Instalar y conectar solo el soporte Tailwind que exige el PMD. Mantener estilos neutros, foco visible y recursos locales. No añadir un sistema visual ni una librería de componentes.

**Acceptance criteria:**
- [x] La integración sigue la documentación oficial compatible con las versiones resueltas de Vite y Tailwind.
- [x] La compilación incluye las clases usadas y conserva la legibilidad y el foco de teclado.
- [x] Los cambios de dependencias quedan reflejados en el lockfile.

**Verification:**
- [x] Prueba enfocada: inspeccionar en la aplicación una clase utilizada y el foco de teclado; no añadir una prueba que replique el CSS.
- [x] Compilación/revisión amplia: `pnpm build`; comprobar instalación con lockfile fijo.
- [x] Revisión manual: abrir con `pnpm tauri dev` y recorrer los controles con teclado.

**Evidence (2026-10-19):** Tailwind y su plugin Vite 4.3.3 instalados como herramientas de desarrollo; Vite 8.3.4 conserva su versión. Integración mediante plugin e import CSS, con utilidades aplicadas a los controles existentes y un contorno `:focus-visible`. `pnpm build` e instalación fija pasan; el lockfile no cambia en la repetición. La app abre en Windows y muestra el espaciado y los estilos compilados; Tab enfoca el primer enlace con contorno visible. Se cerró la ventana y la sesión terminó con código 0. Fuentes: [Tailwind con Vite](https://tailwindcss.com/docs/installation/using-vite) y [directiva apply](https://tailwindcss.com/docs/functions-and-directives#apply-directive).

**Dependencies:** T04, C02
**Files likely touched:** `package.json`, `pnpm-lock.yaml`, `vite.config.ts`, `src/App.css`.
**Estimated scope:** S

## T06: Mostrar un diagnóstico local real

**Status:** Done

**Description:** Sustituir el saludo visible por una vista mínima de Nightingale que lea un comando Rust específico mediante un cliente TypeScript tipado. Mostrar la versión y capacidades locales disponibles; informar las no implementadas sin simular datos ni estados cloud.

**Acceptance criteria:**
- [x] React recibe un diagnóstico real de Rust mediante un comando limitado y tipos explícitos.
- [x] La vista muestra carga, éxito y fallo legible; no muestra secretos, trazas ni rutas personales completas.
- [x] La pantalla identifica qué funciones existen; no presenta sesión, sincronización o citas como implementadas.

**Verification:**
- [x] Prueba enfocada: ejecutar el diagnóstico desde la ventana Tauri y observar la respuesta; comprobar el fallo al usar la vista web sin IPC.
- [x] Compilación/revisión amplia: `pnpm build` y `cargo check --workspace --locked`.
- [x] Revisión manual: `pnpm tauri dev`; solicitar el diagnóstico y operar con teclado.

**Evidence (2026-10-19):** `get_diagnostics` devuelve la versión del paquete Tauri y las constantes de plataforma/arquitectura Rust. Cliente TypeScript tipado, estado explícito de consulta y error sin detalles internos. La pantalla indica que las funciones de negocio siguen pendientes; el documento usa idioma español y título Nightingale. `pnpm build`, `cargo check --workspace --locked` y formato Rust pasan. Cargo informa un aviso de caché incremental que copia archivos al fallar hard links; no es un error del código y el check termina con código 0. En Windows, Tab y Enter sobre el botón muestran `0.1.0`, `windows`, `x86_64` y el mensaje de éxito. En la vista web sin IPC, el botón muestra el error legible previsto. Se cerraron ambas vistas y el proceso de prueba terminó con código 0. Fuentes: [comandos Tauri 2](https://v2.tauri.app/develop/calling-rust/), [AppHandle 2.12.4](https://github.com/tauri-apps/tauri/blob/tauri-v2.12.4/crates/tauri/src/app.rs) y [estado React](https://react.dev/reference/react/useState).

**Dependencies:** T05
**Files likely touched:** `src/App.tsx`, `src/diagnostics.ts`, `src-tauri/src/lib.rs`, `src-tauri/src/diagnostics.rs`, `index.html` (título e idioma). Se conservan los estilos de T05.
**Estimated scope:** M

## T07: Aplicar los límites de seguridad de la base

**Status:** Done

**Description:** Definir la CSP mínima para recursos locales e IPC, limitar permisos a lo usado y excluir secretos y bases de desarrollo de Git. No incorporar acceso cloud ni permisos para funciones futuras.

**Acceptance criteria:**
- [x] La CSP protege la app compilada y permite su IPC; los requisitos del servidor de desarrollo quedan separados.
- [x] La capacidad de la ventana solo concede permisos usados; la base no requiere abrir enlaces externos.
- [x] Git excluye entornos sensibles, bases y archivos auxiliares SQLite; admite ejemplos sintéticos necesarios.

**Verification:**
- [x] Prueba enfocada: `git check-ignore --no-index .env .env.development local.sqlite local.sqlite-wal local.sqlite-shm local.db`; comprobar por separado que un ejemplo sintético puede versionarse.
- [x] Compilación/revisión amplia: `pnpm build`, `cargo check --workspace --locked`; compilación nativa sin empaquetado según el comando fijado en T04.
- [x] Revisión manual: abrir el diagnóstico en desarrollo y en la app compilada; revisar errores CSP sin ampliar permisos por rutina.

**Dependencies:** T06
**Evidence parcial (2026-10-19):** CSP de recursos compilados separada de `devCsp`; conexión IPC local y WebSocket de desarrollo en localhost:1420. `build.rs` declara el único comando permitido mediante `AppManifest::commands`: Tauri permite por defecto los comandos de aplicación en todas las ventanas si no se configura este manifiesto. El archivo generado confirma `allow-get-diagnostics`; la capacidad generada solo lo concede a la ventana local `main`, sin permisos core/opener. Los permisos automáticos se regeneran durante el build y quedan fuera de Git. `git check-ignore --no-index` confirma exclusión de `.env`, `.env.development`, `local.sqlite`, sus archivos wal/shm y `local.db`; `.env.example` no queda excluido. `pnpm build`, `cargo check --workspace --locked`, formato Rust y `git diff --check` pasan. `pnpm tauri build --debug --no-bundle` pasa y genera `target/debug/nightingale.exe` sin instaladores. El proceso de desarrollo llegó a ejecutar el binario; Computer Use se interrumpió con Escape antes de verificar el diagnóstico bajo la nueva CSP. Se detuvo esa sesión y se comprobó que no quedan proceso Nightingale ni escucha en el puerto 1420. Falta la comprobación visual e IPC en desarrollo y en el binario compilado; T07 y C03 siguen pendientes. Fuentes: [capacidades y comandos de aplicación](https://v2.tauri.app/security/capabilities/) y [CSP](https://v2.tauri.app/security/csp/), cotejadas con Tauri 2.12.4 y tauri-build 2.7.3.

**Files likely touched:** `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `.gitignore`, `src-tauri/build.rs` (necesario para aplicar permisos a los comandos propios).
**Estimated scope:** S

## C03: Después de T05–T07

- [x] La vista de Nightingale usa Rust y mantiene accesibilidad básica.
- [x] Funcionan IPC y recursos locales con la CSP; los archivos sensibles quedan fuera de Git.
- [x] Revisión del usuario completa.

**Verificación final de T07 (2026-10-19):** Tras la autorización del usuario para retomar Computer Use, se abrió el ejecutable compilado sin servidor de desarrollo: la vista en http://tauri.localhost/ cargó sus recursos locales, Tab mostró foco y Enter devolvió 0.1.0/windows/x86_64 desde Rust. Después, pnpm tauri dev --no-watch abrió http://localhost:1420/; el mismo botón devolvió el diagnóstico con devCsp. No se observó bloqueo de recursos ni IPC en ambas pruebas. Se cerraron las ventanas; la sesión de desarrollo terminó con código 0. git diff --check pasa. T07 queda completada y C03 queda pendiente solo de revisión del usuario. Evidencia visual: nightingale-compiled-diagnostics.png en la carpeta de artefactos de esta conversación.

**Acceptance (2026-10-19):** El usuario respondió «Si claro, acepto. Podemos continuar». Se acepta C03 y se autoriza T08–T09; C04 mantiene su revisión pendiente.

## T08: Inicializar SQLite con migraciones

**Status:** Done

**Description:** Elegir el adaptador mínimo tras revisar el cifrado futuro y abrir una base vacía en el directorio de datos de la aplicación. Aplicar una migración de metadatos de esquema; rechazar versiones no compatibles y preservar la base ante errores. Mantener las pruebas en archivos del módulo para limitar el cambio.

**Acceptance criteria:**
- [x] Rust crea y reabre una base fuera del repositorio con migraciones versionadas; React no ejecuta SQL.
- [x] Las pruebas con rutas temporales prueban creación, reapertura sin repetir la migración y rechazo no destructivo de esquema no compatible/fallo.
- [x] La app informa el fallo de inicialización sin borrar ni reemplazar una base existente y no introduce tablas de negocio ni datos reales.

**Verification:**
- [x] Prueba enfocada: `cargo test --workspace --locked` con los casos temporales añadidos al módulo SQLite.
- [x] Compilación/revisión amplia: `cargo check --workspace --locked` y `pnpm build`.
- [x] Revisión manual: abrir, cerrar y volver a abrir la app; comprobar que conserva el esquema y no escribe la base en el repositorio.

**Dependencies:** T07, C03
**Files likely touched:** `src-tauri/Cargo.toml`, `Cargo.lock`, `src-tauri/src/lib.rs`, `src-tauri/src/sqlite.rs`, `src-tauri/migrations/0001_schema.sql`.
**Estimated scope:** M

**Evidence (2026-10-19):** rusqlite 0.40.2 con SQLite compilado incluido, sin caché ni soporte WASM innecesarios; libsqlite3-sys 0.38.2 en Cargo.lock. Se cotejó el soporte futuro SQLCipher en su README oficial: permite cambiar la biblioteca, pero no resuelve claves ni migración de datos cifrados; esas decisiones siguen abiertas antes de usar datos reales. Rust abre nightingale.sqlite mediante app_data_dir de Tauri 2.12.4, conserva la conexión y rechaza la inicialización fallida con un mensaje de arranque sin rutas ni datos del archivo. Migración 0001_schema transaccional con identificador de aplicación, user_version e historial; no hay tablas de negocio. Siete pruebas temporales pasan con cargo test --workspace --locked: creación/reapertura sin cambios, esquema futuro, base ajena, historial inválido, corrupción, rollback y fallo de apertura. pnpm build, cargo check --workspace --locked, formato Rust y git diff --check pasan. Apertura/cierre/reapertura Windows comprobados: base fuera del repositorio, versión 1, único registro 0001_schema y SHA256 idéntico AF46DCD749E80B0D1725E94246D951BDA0974015CC21BC28A71746AD66663249. Ambas sesiones nativas terminan con código 0. Fuentes: https://github.com/rusqlite/rusqlite/blob/v0.40.2/README.md ; https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior ; https://github.com/tauri-apps/tauri/blob/tauri-v2.12.4/crates/tauri/src/path/desktop.rs .

## T09: Unificar los controles de calidad

**Status:** Done

**Description:** Crear un comando raíz que ejecute las comprobaciones de tipos/compilación web, formato Rust, análisis Rust y pruebas reales de persistencia. Definir también el comando de compilación nativa sin publicación. Usar herramientas presentes en el stack salvo necesidad probada.

**Acceptance criteria:**
- [x] Un comando funciona con la misma intención en PowerShell y los shells de macOS/Linux, propaga fallos y no oculta comprobaciones ausentes.
- [x] Ejecuta las pruebas SQLite, tipos/compilación web y controles Rust sin modificar archivos fuente ni lockfiles.
- [x] Quedan documentados su composición y el comando exacto de compilación nativa sin empaquetar/publicar.

**Verification:**
- [x] Prueba enfocada: `pnpm quality`; ejecutar el comando nuevo y comprobar su código de salida ante un fallo temporal controlado, que se revierte antes de terminar.
- [x] Compilación/revisión amplia: `pnpm build`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`.
- [x] Revisión manual: ejecutar desde raíz y verificar que el colaborador no necesita conocer directorios internos.

**Dependencies:** T08
**Files likely touched:** `package.json`, `README.md`; configuración de compilación solo si hace falta y con un máximo de cinco archivos.
**Estimated scope:** S

## C04: Después de T08–T09

- [x] Pasan las pruebas de creación, reapertura y rechazo no destructivo de SQLite.
- [x] El comando común pasa y detecta fallos reales.
- [x] Revisión del usuario completa.

**Evidence de T09 y C04 (2026-10-19):** pnpm quality ejecuta pnpm build, cargo fmt --all -- --check, cargo clippy --workspace --all-targets --locked -- -D warnings y cargo test --workspace --locked mediante una cadena que se detiene al primer fallo. Pasa en Windows, incluidas siete pruebas SQLite. Una prueba temporal añadió un error TypeScript real: quality devolvió 2 y no continuó a Rust; el archivo temporal se eliminó y quality volvió a pasar. La comparación SHA256 antes/después confirma que quality no cambia fuentes, manifiestos ni lockfiles. En revisión se corrigió el filtro de objetos internos SQLite para rechazar una tabla ajena sqlitex; su prueba pasa y quality se repitió tras el cambio. pnpm build:desktop ejecuta tauri build --debug --no-bundle y pasa con la revisión final, sin generar instaladores ni publicar. README documenta ambos comandos y las pruebas nativas pendientes en macOS/Linux. git diff --check pasa y no quedan procesos Nightingale abiertos. Fuentes: https://pnpm.io/cli/run ; https://doc.rust-lang.org/clippy/usage.html ; https://v2.tauri.app/reference/cli/#build . C04 requiere aceptación del usuario antes de T10.

**Acceptance (2026-10-19):** El usuario respondió «Excelente, continuemos entonces». Se acepta C04 y se inicia T10; C05 conserva todas las pruebas y la aceptación final pendientes.

## T10: Comprobar los tres sistemas en CI

**Status:** Done

**Cierre por retirada (2026-10-21):** el usuario pidió eliminar los jobs. Este cierre registra su retirada, no una validación completa de CI. Los criterios y evidencias originales que siguen son históricos y ya no bloquean C05. Se elimina el workflow y se mantienen los controles locales.

**Description original:** Añadir una matriz de CI con hosts nativos para la cobertura fijada en T02. Instalar requisitos del sistema, usar herramientas y lockfiles acordados y ejecutar los controles de T09 y la compilación del escritorio. No copiar los pasos de publicación de una guía de releases.

**Acceptance criteria originales (retirados):**
- [x] La matriz cubre Windows, macOS y Linux con las arquitecturas acordadas y dependencias nativas explícitas.
- [ ] Cada host ejecuta instalación fija, controles de calidad y compilación del escritorio; no requiere credenciales cloud ni publica releases.
- [x] La evidencia corresponde al mismo cambio revisado; si no se pueden ejecutar jobs remotos, se marca la verificación pendiente.

**Verification original (retirada):**
- [x] Prueba enfocada: revisar cada job contra los comandos de T09 y la guía oficial Tauri 2 para su sistema.
- [ ] Compilación/revisión amplia: resultados de la matriz completa; el comando exacto procede de T09, no de una suposición del plan.
- [ ] Revisión manual: consultar los resultados y confirmar que ningún job omite una plataforma o necesita un secreto privado.

**Dependencies:** T09, C04
**Files likely touched:** `.github/workflows/quality.yml`.
**Estimated scope:** S

**Decisión del usuario (2026-10-19):** «Deja eso como pendiente, yo lo configuraré por mi mismo». El propietario configurará el remoto; no añadirlo ni publicar por deducción. La ejecución y revisión remotas siguen pendientes por decisión explícita.

**Evidence parcial (2026-10-19):** Workflow creado para windows-2025, macos-15 ARM64, macos-15-intel y ubuntu-24.04. Cada job instala Node desde .node-version, pnpm 11.17.0 y el toolchain Rust desde rust-toolchain.toml; usa instalación fija, pnpm quality, pnpm build:desktop y comprueba que no cambien archivos versionados. Linux instala requisitos Tauri; Windows usa C++/SDK de su imagen y macOS comprueba herramientas Xcode. Sin publicación ni secretos privados, con contents:read y sin persistir credenciales del checkout. YAML leído con un parser y revisado contra documentación oficial de Tauri, imágenes/etiquetas GitHub y action.yml de checkout/setup-node v7. git remote -v sigue vacío: no hay jobs remotos ni resultados que revisar. Se solicitó la URL al usuario; no se inventó ni añadió un remoto. T10 queda bloqueada solo en su ejecución y revisión remotas. Se prepara la documentación independiente de T11 sin marcar aprobadas esas pruebas. Fuentes: https://v2.tauri.app/start/prerequisites/ ; https://docs.github.com/en/actions/reference/runners/github-hosted-runners ; https://github.com/actions/setup-node/tree/v7 ; https://github.com/actions/checkout/tree/v7 .

## T11: Documentar la incorporación de colaboradores

**Status:** Done

**Cierre (2026-10-21):** el usuario pidió «Por favor, marca esas como listas». Se registra su aceptación para cerrar las comprobaciones pendientes sin afirmar que se realizaron. La evidencia anterior conserva su fecha y describe el estado previo a esta decisión.

**Description:** Sustituir el README de la plantilla por instrucciones probadas: requisitos por sistema, versiones, instalación, arranque, calidad, ubicación de datos, solución de fallos y límites de la base. Añadir una guía breve para contribuciones y para las rutas futuras sin planificar servicios web ahora.

**Acceptance criteria:**
- [x] Un clon limpio permite seguir la guía sin configuración privada; constan instalación fija, comandos exactos y errores frecuentes.
- [x] Se explica dónde colocar interfaz, dominio, adaptadores y pruebas cuando exista una función, sin crear carpetas ceremoniales ni nuevos contratos de negocio.
- [x] Cierre aceptado por el usuario sin revisión de otro colaborador ni pruebas manuales en Fedora/macOS. Estas comprobaciones no se realizaron y no se registran como aprobadas técnicamente.

**Verification:**
- [x] Prueba enfocada: seguir las instrucciones desde un clon limpio con datos sintéticos; no reutilizar `node_modules`, bases o archivos privados del autor.
- [x] Compilación/revisión amplia: calidad y compilación verificadas en Windows y en la ejecución histórica de los cuatro entornos; documentos comprobados con `git diff --check`. El usuario acepta el cierre sin nuevas pruebas locales en Fedora/macOS.
- [x] Revisión manual: apertura, diagnóstico y reapertura comprobados en Windows. El usuario acepta sin verificación manual de Fedora/macOS el 2026-10-21; no consta su ejecución.

**Dependencies:** T10
**Files likely touched:** `README.md`, `CONTRIBUTING.md`, `docs/operations/development.md`.
**Estimated scope:** M

**Evidence parcial (2026-10-19):** README sustituido; CONTRIBUTING y docs/operations/development.md creados con requisitos por sistema, versiones, comandos, rutas, datos locales, fallos y límites. Enlaces locales y bloques Markdown revisados. Clon local limpio del snapshot temporal 2dc472583eee99667f620db5275760920b7f6b16 del trabajo actual: sin copiar node_modules, dist, target, bases ni configuración privada; usa los almacenes de paquetes normales del host. El snapshot y el clon están bajo target/team-check, fuera de los archivos versionados del proyecto principal; no se creó un commit en el repositorio principal. Windows 11 Home 10.0.26200 x64, Node 24.18.0, pnpm 11.17.0 y Rust 1.99.1. pnpm install --frozen-lockfile, rustup show, pnpm quality y pnpm build:desktop pasan desde el clon. Las siete pruebas SQLite pasan; Clippy no emite avisos. git diff --exit-code y git status --porcelain del clon quedan limpios. Para la apertura se usó --config con un identificador temporal com.pedro.nightingale.onboarding-check, admitido por la CLI 2.12.3; no se modificaron fuentes ni el identificador del proyecto. Se comprobó que su base no existía, luego se abrió la ventana, Tab mostró foco y Enter devolvió 0.1.0/windows/x86_64. Cierre/reapertura conserva el hash de la nueva base y su único registro 0001_schema; la base anterior también conserva su hash. Ambas sesiones terminan con código 0 y no quedan procesos ni puerto 1420 abierto. Captura: nightingale-clean-clone.png en artefactos de esta conversación. El primer git clone falló por el proceso MSYS bajo sandbox; al repetir con acceso permitido pasó, sin modificar código. Faltan CI, aperturas macOS/Linux y la revisión de otro colaborador: T11 y C05 no se declaran completos. Fuente del perfil temporal: https://v2.tauri.app/reference/cli/#dev y ayuda local de la CLI instalada.

## C05: Después de T10–T11

- [x] Instalación fija, calidad y compilación con evidencia Windows y CI histórica en los cuatro entornos. El usuario acepta esta evidencia para el cierre; no se afirma una prueba local nueva en Fedora/macOS.
- [x] Apertura y guía aceptadas por el usuario: Windows verificado; pruebas manuales Fedora/macOS y revisión de otro colaborador aceptadas sin verificación, no realizadas.
- [x] El usuario acepta la base el 2026-10-21 al pedir cerrar estos puntos. Esta aceptación no declara implementadas citas, sincronización ni servicios web.

**Aceptación del cierre (2026-10-21):** El remoto está configurado y el usuario retiró los jobs de CI. Después pidió marcar listas las comprobaciones restantes. C05 se cierra por su aceptación; las aperturas Fedora/macOS y la revisión de otro colaborador no se realizaron y no se convierten en resultados de prueba. La tabla distingue esa aceptación de la evidencia técnica.

| Entorno | Instalación / calidad / compilación | Apertura / diagnóstico / reapertura | Revisión de otro colaborador |
|---|---|---|---|
| Windows 11 x64, clon local del snapshot indicado | Pasa | Pasa con perfil temporal nuevo; la configuración normal se probó en T07/T08 | Aceptada sin verificación |
| Windows en CI (histórico) | Pasó en 632f565; jobs retirados | No sustituye apertura en Windows 11 | Aceptada sin verificación |
| macOS 15 ARM64 | Pasó en CI para 632f565; jobs retirados | Aceptada sin verificación | Pendiente |
| macOS 15 Intel | Pasó en CI para 632f565; jobs retirados | Aceptada sin verificación | Pendiente |
| Fedora 44 x64 | Instalación/calidad/compilación pasaron en CI para 632f565; último control Git falló; jobs retirados | No realizada; aceptada sin verificación | Aceptada sin verificación |

**Corrección posterior del usuario (2026-10-19):** «Por ahora, ninguno y de hecho usamos Fedora 44 y no Ubuntu». Fedora 44 sustituye Ubuntu como Linux de desarrollo y validación. La configuración anterior de T10 se reemplaza por tres hosts Windows/macOS y un job en contenedor oficial fedora:44 sobre ubuntu-24.04; las dependencias se instalan con dnf y rustup carga el toolchain fijado. README, guía y ADR reflejan Fedora. El contenedor no prueba apertura gráfica. No hay Docker/Podman disponible en este host Windows para ejecutarlo; Fedora, macOS, CI y revisión de otro colaborador conservan su estado pendiente. La prueba Windows del snapshot previo sigue probando el código de producto, que no cambia en esta corrección; no prueba el workflow Fedora nuevo.

**Comprobaciones de la corrección Fedora (2026-10-19):** YAML parseado; tres entradas Windows/macOS más un job Fedora 44. Ese job usa instalación fija, quality y build:desktop, sin apt ni secretos privados. `sh -n` pasa sobre sus comandos sin ejecutarlos. Enlaces locales y `git diff --check` pasan. Código de producto, manifiestos y lockfiles coinciden con el snapshot Windows probado; no se repitió una compilación Windows que no verifica este cambio. No se ha ejecutado dnf, el contenedor ni una ventana Fedora: sus resultados siguen pendientes. La lista Tauri y el catálogo Fedora 44 confirman WebKitGTK 4.1, AppIndicator, librsvg y libxdo; la imagen oficial publica la etiqueta 44 x64. No se retira macOS del alcance.

**Evidencia remota y retirada (2026-10-21):** origin apunta a https://github.com/odiseo0/nightingale.git. La ejecución https://github.com/odiseo0/nightingale/actions/runs/39342249145 corresponde al commit 632f565fe2ed7b78e7bb71049c7c5ec6c8128b8e. La API pública confirma instalación fija, quality y build:desktop correctos en los cuatro entornos; Windows y ambos Mac terminan con éxito. Fedora falla solo en Check tracked files stay unchanged, código 129. El registro detallado requiere autenticación (API 403); no se atribuye una causa sin leerlo. El usuario optó por retirar los jobs. Se elimina el archivo del workflow local; no se borran ejecuciones históricas ni se publica el cambio por deducción. C05 continúa abierto por las pruebas manuales y revisión pendientes.

**Cierre final por el usuario (2026-10-21):** «Por favor, marca esas como listas». T11 y C05 se cierran por aceptación explícita. Las notas previas sobre bloqueos describen el estado histórico; las pruebas manuales Fedora/macOS y la revisión independiente siguen sin evidencia de ejecución y dejan de ser requisitos de cierre por esta decisión. No se han ejecutado pruebas nuevas ni publicado cambios como parte de esta actualización.
