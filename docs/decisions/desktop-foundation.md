# ADR: Base de desarrollo del escritorio

**Estado:** Aceptada en C01 por el usuario el 2026-10-19: «Puedes continuar, por favor».
**Fecha:** 2026-10-19
**Corrección de Linux (2026-10-19):** el usuario indicó «usamos Fedora 44 y no Ubuntu». Fedora 44 reemplaza la referencia Ubuntu acordada antes; se conserva la aceptación histórica de C01. El usuario no dispone ahora de los equipos para las pruebas pendientes.
**Plan:** [P-20261019-01-desktop-foundation](../../tasks/P-20261019-01-desktop-foundation/plan.md)
**Reemplaza:** ninguna decisión previa. Concreta las rutas conceptuales del PMD §8 y la cobertura de desarrollo del escritorio.

**Vigencia arquitectónica (2026-10-21):** la [decisión de operación nativa y servicios cloud](native-desktop-cloud-boundaries.md) actualiza la propiedad de componentes y los límites entre repositorios. Este ADR conserva las decisiones de herramientas, rutas y plataformas de la base; sus menciones al servidor describen el alcance histórico y no exigen una API central.

**Revisión arquitectónica posterior (2026-10-21):** la [decisión de tres proyectos y API transaccional](transactional-sync-three-projects.md) sustituye la decisión citada arriba. El escritorio mantiene Rust/SQLite local y sincroniza por HTTPS con Nightingale API, sin acceso SQL remoto. Nightingale Patient Hub tiene su propio proyecto y usa Nightingale API para operar sobre citas. Este cambio documental conserva las herramientas, plataformas y evidencia histórica de la base.

## Contexto

El usuario confirmó Windows, macOS y Linux desde el inicio. La base actual contiene una plantilla Tauri con React y TypeScript en raíz. Los servicios web se abordarán después, cuando los colaboradores tomen sus rutas de trabajo.

El manifiesto Cargo raíz no tiene un target válido y bloquea también la carga del manifiesto de Tauri. No hay lockfile JavaScript ni versiones de herramientas declaradas para el equipo. Esta decisión fija la base que se aplicará en T03–T04; todavía no demuestra que compile.

**Vigencia de CI (2026-10-21):** el usuario retiró los jobs de GitHub Actions. Las referencias a hosts y contenedores CI que siguen registran la decisión anterior; ya no describen una obligación ni un workflow presente. La cobertura de desarrollo sigue siendo Windows 11 x64, macOS 15 ARM/Intel y Fedora 44 x64, con los controles locales y pruebas manuales.

## Decisión

### 1. Conservar las rutas útiles de la plantilla

| Ruta actual | Responsabilidad | Límite |
|---|---|---|
| `package.json`, `vite.config.ts`, `index.html` | Construcción y entrada de la interfaz de escritorio | No representan un servidor web del producto. |
| `src/` | Interfaz React del escritorio y cliente Tauri tipado | Sin SQL, secretos cloud o reglas definitivas de disponibilidad. |
| `src-tauri/` | Proceso Rust del escritorio | Composición Tauri; reglas locales y adaptadores cuando una función los necesite. |
| `src-tauri/src/` | Código Rust actual | El saludo de plantilla no es un componente de negocio. Crear módulos cuando tengan comportamiento, no directorios vacíos. |
| `Cargo.toml`, `Cargo.lock` en raíz | Workspace Rust y resolución común de dependencias | T03 hará a `src-tauri` miembro de un workspace virtual. No obliga a futuros servidores a usar Rust. |
| `docs/` | Requisitos, arquitectura, decisiones y operación | Los cambios de alcance requieren una decisión explícita. |
| `tasks/` | Planes, tareas, comprobaciones y revisiones | No indica que una función de la SRS ya esté implementada. |

El árbol de PMD §8 sigue definiendo responsabilidades. Para esta base, `src/` corresponde a su interfaz de escritorio y `src-tauri/` a su núcleo local. Se conserva esta ubicación para no mover fuentes, recursos e iconos que ya reconoce Tauri.

Cuando haya trabajo web aprobado, se podrán incorporar `apps/`, `services/` y `contracts/` según el PMD. Ese trabajo decidirá los workspaces JavaScript necesarios. No crear ahora otra aplicación, un servicio vacío ni un workspace JavaScript para aplicaciones inexistentes.

### 2. Fijar herramientas conocidas

| Herramienta | Versión propuesta | Aplicación posterior | Motivo y evidencia |
|---|---|---|---|
| Node.js | `24.18.0` | `.node-version` y declaración de compatibilidad en T04 | Versión LTS publicada e instalada en el host local. Cumple `^20.19.0 || >=22.12.0`, requerido por Vite 8.0.16 y el plugin React 6.0.2. |
| pnpm | `11.17.0` | `packageManager` y lockfile en T04 | Release publicada y disponible en el host local. Los hooks Tauri ya llaman a pnpm; conservar el gestor evita cambiar el flujo. |
| Rust/Cargo | `1.99.1` | `rust-toolchain.toml` en T03, con `rustfmt` y `clippy` | Release publicada y disponible en el host local. Soporta la edición 2024 que usan los manifiestos; no seguir el canal móvil `stable`. |

Estas versiones son una base fija para reproducir el trabajo, no una afirmación de que sean las últimas. Toda actualización deberá repetir instalación, calidad y compilación en la matriz cubierta. No actualizar versiones principales de React, Vite, TypeScript o Tauri como efecto de fijar herramientas.

Los manifiestos actuales declaran React `^19.1.0`, React DOM `^19.1.0`, TypeScript `~6.0.3`, Vite `^8.0.16`, plugin React `^6.0.2` y Tauri `^2`/`2`. Los rangos no son versiones resueltas. T04 generará el lockfile JavaScript y T03 el lockfile Rust válido; sus resultados deberán comprobarse antes de declarar compatible la instalación completa.

### 3. Cubrir cuatro entornos

La propuesta de referencia para desarrollo es Windows 11 x64, macOS 15 en Apple Silicon e Intel, y Fedora 44 x64. Son entornos de prueba de la base, no una promesa de instaladores para todas las versiones de cada sistema.

| Entorno de desarrollo | Target Rust nativo | Host CI propuesto | Requisitos nativos principales |
|---|---|---|---|
| Windows 11 x64 | `x86_64-pc-windows-msvc` | `windows-2025` | C++ Build Tools con desarrollo de escritorio C++, Windows SDK y WebView2. |
| macOS 15 Apple Silicon | `aarch64-apple-darwin` | `macos-15` | Xcode Command Line Tools y sus herramientas de compilación. |
| macOS 15 Intel | `x86_64-apple-darwin` | `macos-15-intel` | Xcode Command Line Tools y sus herramientas de compilación. |
| Fedora 44 x64 | `x86_64-unknown-linux-gnu` | Contenedor `fedora:44` sobre `ubuntu-24.04` | Compilador, `pkg-config`, WebKitGTK 4.1, OpenSSL, librsvg, libxdo y AppIndicator según la guía Tauri. |

El host Windows de CI usa Windows Server 2025: su compilación no sustituye la apertura manual en Windows 11. Cada Mac compila de forma nativa para su arquitectura; no se necesita una app universal en esta base. En Linux se compila y prueba el código en el entorno Fedora 44 del contenedor con glibc; Ubuntu solo aporta el host Docker. Este job no sustituye la apertura manual en un escritorio Fedora 44 con Wayland/X11; no se afirma soporte de todas las distribuciones ni de musl. Windows/Linux ARM y sistemas anteriores quedan fuera de esta matriz propuesta, no del posible alcance futuro.

Cada entorno deberá instalar desde lockfiles, ejecutar los controles comunes y compilar el escritorio. La apertura nativa se anotará por separado, con sistema y arquitectura. Un job verde sin apertura no prueba el uso de la interfaz. Firmas, notarización, instaladores y publicación quedan fuera de esta base.

### 4. Usar comandos desde raíz

| Comando | Estado actual | Uso |
|---|---|---|
| `pnpm install --frozen-lockfile` | Pendiente de T04; todavía falta el lockfile | Instalación repetible una vez generado y revisado el lockfile. |
| `pnpm dev` | Script existente; falta instalar dependencias | Vista web de desarrollo. No prueba el IPC nativo por sí sola. |
| `pnpm build` | Script existente; falta instalar dependencias | Tipos y construcción de recursos web. |
| `pnpm tauri dev` | Script existente; pendiente de T03–T04 y requisitos nativos | Abrir el escritorio y probar React → Rust. |
| `cargo metadata --no-deps --format-version 1` | Cargo lo admite; hoy falla por el manifiesto raíz | Confirmar que Cargo carga el workspace tras T03. |
| `cargo check --workspace --locked` | Pendiente de T03 | Comprobar Rust con resolución fija. |
| Comando común de calidad | Pendiente de T09 | Ejecutar tipos, compilación web, formato, análisis Rust y pruebas de persistencia. No documentar un script inexistente como disponible. |
| Compilación nativa sin empaquetar | Pendiente de T04 | Confirmar la opción exacta con la CLI Tauri instalada; reutilizarla en T07, T09 y CI. |

No instalar PostgreSQL, n8n, una cola o herramientas del servidor para ejecutar esta base. La ausencia de esos servicios no debe impedir abrir la app local ni ejecutar sus pruebas.

## Alternativas consideradas

- **Mover ya todo a `apps/desktop/`:** refleja el árbol conceptual, pero obliga a cambiar rutas y recursos sin habilitar una función. Se conserva el mapa anterior para esta base.
- **Eliminar toda coordinación Rust y usar solo `src-tauri/Cargo.toml`:** puede resolver la carga local si se elimina el manifiesto inválido, pero pierde los comandos y el lockfile comunes previstos en el plan aprobado. Se elige un workspace virtual mínimo con un solo miembro real.
- **Usar herramientas móviles como `latest` o `stable`:** reduce la configuración inicial, pero permite cambios de compilador o gestor entre colaboradores. Se fijan versiones concretas.
- **Comprobar solo Windows y ampliar después:** contradice la petición del usuario. Se incluyen los tres sistemas desde la base y las dos arquitecturas comunes de Mac.
- **Elegir ahora el servidor para completar el árbol:** amplía el alcance inicial aclarado por el usuario. Se deja para su ruta posterior.

## Consecuencias y límites

- Un colaborador conoce qué fuentes pertenecen al escritorio sin una mudanza inicial.
- Los cuatro entornos usarán las mismas versiones de herramientas; las dependencias opcionales nativas aún se deben comprobar en T10.
- La instalación inicial necesita red y acceso al almacén de paquetes. Una falla de permisos no demuestra un defecto del código.
- `git remote -v` no devuelve un remoto configurado en esta revisión. T10 puede añadir el workflow, pero ejecutar y comprobar CI requerirá un repositorio remoto y resultados reales. No inventar una dirección ni publicar el trabajo por deducción.
- No hay evidencia de compilación completa o apertura en los cuatro entornos. T03–T11 deben aportarla; esta decisión no la sustituye.
- El usuario aceptó en C01 las rutas, versiones y cobertura descritas aquí antes de iniciar T03.

## Fuentes oficiales consultadas

Comprobadas el 2026-10-19:

- [Node.js 24.18.0 LTS](https://nodejs.org/en/blog/release/v24.18.0): release y binarios para los hosts propuestos.
- [pnpm 11.17.0](https://github.com/pnpm/pnpm/releases/tag/v11.17.0): release fijada; no usar instrucciones exclusivas de pnpm 12 para configurar pnpm 11.
- [Rust 1.99.1](https://github.com/rust-lang/rust/blob/1.99.1/RELEASES.md): release del compilador; [edición 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html), disponible desde Rust 1.85.
- [Vite 8.0.16, manifiesto oficial](https://github.com/vitejs/vite/blob/v8.0.16/packages/vite/package.json): requisito Node.
- [Plugin React 6.0.2, manifiesto oficial](https://github.com/vitejs/vite-plugin-react/blob/plugin-react%406.0.2/packages/plugin-react/package.json): Node y peer Vite `^8.0.0` compatibles con la propuesta.
- [Cargo: workspace virtual](https://doc.rust-lang.org/cargo/reference/workspaces.html#virtual-workspace): raíz sin paquete y miembro explícito; resolver explícito y perfiles en raíz.
- [Tauri 2: requisitos](https://v2.tauri.app/start/prerequisites/): herramientas nativas de Windows, macOS y Linux; [guía Vite](https://v2.tauri.app/start/frontend/vite/): separación entre recursos web y proceso Tauri.
- [GitHub: hosts disponibles](https://docs.github.com/en/actions/reference/runners/github-hosted-runners): etiquetas Windows, Ubuntu y Mac Intel/ARM; [Tauri 2: CI](https://v2.tauri.app/distribute/pipelines/github/): compilación en hosts nativos. No copiar publicación de releases a este plan.

### Fuentes de la corrección Fedora 44

- [Tauri 2: requisitos Linux/Fedora](https://v2.tauri.app/start/prerequisites/#linux).
- [WebKitGTK 4.1 en Fedora 44](https://packages.fedoraproject.org/pkgs/webkitgtk/webkit2gtk4.1-devel/fedora-44.html) y [libappindicator GTK3](https://packages.fedoraproject.org/pkgs/libappindicator/libappindicator-gtk3-devel/fedora-44.html).
- [Imagen oficial Fedora: etiqueta 44](https://github.com/docker-library/official-images/blob/master/library/fedora).
- [GitHub: jobs en contenedores](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/run-jobs-in-a-container).
- [rustup: instalación sin toolchain por defecto](https://rust-lang.github.io/rustup/installation/index.html#installing-nightly); el archivo del proyecto elige después Rust 1.99.1.
