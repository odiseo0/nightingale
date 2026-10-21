# Nightingale

Aplicación de escritorio para gestionar citas y reducir ausencias en consultas médicas. Esta base prepara el trabajo del equipo: React llama a Rust mediante Tauri y Rust abre una base SQLite local con una migración de metadatos.

Todavía no hay gestión de pacientes o citas, sesión, sincronización ni servicios web. No introduzcas datos reales: falta definir el cifrado y las claves.

## Arquitectura objetivo

Rust ejecutará los casos de uso del escritorio sobre SQLite sin depender de HTTP. Su coordinador enviará operaciones completas por HTTPS a Nightingale API cuando exista conexión. Esa API autorizará y revalidará cada operación antes de confirmar su transacción en PostgreSQL privado. El escritorio no tendrá acceso SQL ni credenciales de esa base.

El sistema tendrá tres proyectos con repositorios propios:

| Proyecto | Responsabilidad |
|---|---|
| **Nightingale** | Este escritorio: Tauri, React, Rust, SQLite y sincronización local. |
| **Nightingale Patient Hub** | Web y API propia; enlaces y acciones del paciente sobre citas existentes. Su backend llama a Nightingale API para consultar y modificar citas. |
| **Nightingale API** | Python/FastAPI, aceptación transaccional, sincronización y PostgreSQL privado; módulos y workers de recordatorios, integración con n8n y analítica. |

Rust consumirá la API del Hub para sus capacidades y conservará una vista local versionada. Los workers ejecutarán trabajo externo después del commit, sin bloquear transacciones de sincronización. n8n seguirá siendo una herramienta externa.

Sin conexión se usarán los últimos permisos válidos; cambiarlos requiere conexión. El [ADR vigente y su diagrama](docs/decisions/transactional-sync-three-projects.md) y las especificaciones breves de [Nightingale API](docs/specifications/nightingale-api.md) y [Patient Hub](docs/specifications/nightingale-patient-hub.md) recogen el diseño. Los otros dos repositorios se crearán aparte. Son objetivos del producto; la base disponible sigue limitada al diagnóstico y los metadatos SQLite descritos arriba.

## Empezar

1. Instala los [requisitos de tu sistema](docs/operations/development.md#requisitos).
2. Usa Node.js **24.18.0**, pnpm **11.17.0** y Rust **1.99.1**.
3. Desde la raíz del repositorio, ejecuta:

```sh
pnpm install --frozen-lockfile
rustup show
pnpm quality
pnpm tauri dev
```

En la ventana, pulsa **Consultar diagnóstico**. Debe mostrar la versión, el sistema y la arquitectura que devuelve Rust. Cierra la ventana para detener la sesión. La primera compilación puede tardar mientras Cargo descarga y compila.

No necesitas PostgreSQL, Docker, credenciales cloud ni archivos `.env`. La instalación inicial necesita red. La [guía de desarrollo](docs/operations/development.md) incluye instalación de herramientas, datos locales y solución de fallos.

## Comandos desde la raíz

| Comando | Resultado |
|---|---|
| `pnpm install --frozen-lockfile` | Instala las dependencias JavaScript fijadas. |
| `pnpm tauri dev` | Abre el escritorio con recarga de desarrollo. |
| `pnpm dev` | Abre solo el servidor web; no ofrece IPC con Rust. |
| `pnpm quality` | Tipos y compilación web, formato Rust, Clippy y pruebas SQLite. Se detiene al primer fallo. |
| `pnpm build:desktop` | Compila el escritorio en debug con recursos locales, sin instaladores ni publicación. |
| `cargo test --workspace --locked` | Ejecuta las pruebas Rust con bases temporales aisladas. |

## Dónde trabajar

| Ruta | Uso |
|---|---|
| `src/` | Interfaz React y cliente IPC tipado; sin SQL. |
| `src-tauri/src/` | Composición Tauri, comandos y acceso local desde Rust. |
| `src-tauri/migrations/` | Migraciones SQLite ordenadas. |
| `docs/` | Requisitos, arquitectura, decisiones y operación. |
| `tasks/` | Planes aprobados, tareas y evidencia de sus comprobaciones. |

Lee [CONTRIBUTING.md](CONTRIBUTING.md) antes de tomar una ruta. El [Master Plan](docs/master-plan.md) define el producto; el [ADR del escritorio](docs/decisions/desktop-foundation.md) fija herramientas y cobertura. Los servicios web tendrán sus propios planes.

## Estado de verificación

La instalación, calidad, compilación y apertura se han probado en Windows x64. La base contempla Windows x64, macOS 15 ARM64/Intel y Fedora 44 x64. El usuario aceptó el cierre de la base el 2026-10-21 sin las pruebas manuales macOS/Fedora ni la revisión de otro colaborador. Estas pruebas no se realizaron; la aceptación no equivale a una verificación técnica. Consulta la evidencia en [C05](tasks/P-20261019-01-desktop-foundation/todo.md#c05-después-de-t10t11).

El equipo ejecuta los controles desde la terminal. Por decisión del propietario del 2026-10-21, esta base no incluye jobs de GitHub Actions. Se conservan `pnpm quality` y `pnpm build:desktop`. No se han preparado firmas, instaladores, actualizaciones ni releases.
