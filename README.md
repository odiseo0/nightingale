# Nightingale

Aplicación de escritorio para gestionar citas y reducir ausencias en consultas médicas. Esta base prepara el trabajo del equipo: React llama a Rust mediante Tauri y Rust abre una base SQLite local con una migración de metadatos.

Todavía no hay gestión de pacientes o citas, sesión, sincronización ni servicios web. No introduzcas datos reales: falta definir el cifrado y las claves.

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
