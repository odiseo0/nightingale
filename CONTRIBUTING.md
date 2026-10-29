# Contribuir a Nightingale

Empieza por el [README](README.md), la [guía de desarrollo](docs/operations/development.md) y las partes del [Master Plan](docs/master-plan.md) que afecten a tu trabajo. Sigue también [AGENTS.md](AGENTS.md) si trabajas con un agente.

## Tomar una tarea

Acuerda con el equipo la ruta y su responsable antes de cambiar contratos compartidos. Consulta los planes en `tasks/` para evitar trabajo duplicado. Un cambio amplio necesita un plan revisado antes de implementar; usa las [reglas de nombres](tasks/README.md), conserva los planes previos y registra las comprobaciones de cada tarea. No interpretes una función descrita en la SRS como una función ya construida.

Trabaja en una rama propia. Si usas Codex, el prefijo previsto es `codex/`. Conserva los cambios ajenos del directorio de trabajo. No actualices herramientas o dependencias como parte de otro cambio sin explicar por qué hace falta.

## Colocar el código

- Interfaz y estados de vista: `src/`. Añade el cliente IPC tipado en `src/shared/api/`; no pases SQL, secretos o acceso general al sistema de archivos.
- Entrada y composición del escritorio: `src-tauri/src/lib.rs`. Mantén aquí la conexión entre Tauri y los módulos; evita acumular reglas del producto.
- Contratos compartidos mínimos: `src-tauri/src/shared_kernel/`. No añadas entidades, repositorios o clientes HTTP al kernel.
- Infraestructura sin reglas de producto: `src-tauri/src/platform/`. La apertura y migraciones SQLite viven en `platform/sqlite/`.
- Casos de uso: `src-tauri/src/components/<componente>/`, separando dominio, aplicación y adaptadores solo cuando exista comportamiento que los necesite. Prueba entradas, salidas y persistencia con bases temporales.
- Migraciones: archivos numerados en `src-tauri/migrations/`. No edites una migración ya usada para cambiar su sentido. Reserva el número en el `todo.md` del plan y prueba éxito y fallo sin pérdida de datos.

Cada componente es dueño de sus tablas; los demás consumen sus puertos y no ejecutan SQL sobre ellas. El núcleo Rust opera sobre SQLite sin HTTP; el coordinador sincroniza operaciones completas mediante Nightingale API por HTTPS. La API revalida y acepta el estado compartido; no entregues acceso SQL o credenciales PostgreSQL al escritorio. El [ADR del motor transversal](docs/decisions/desktop-core-engine.md) fija propietarios, estados y recuperación.

Antes de repartir trabajo paralelo, congela DTO, acciones, tablas, proyectores, comandos y reservas de migración en el plan. El integrador nombrado registra los puntos calientes: `lib.rs`, `build.rs`, `components/mod.rs`, `platform/sqlite/mod.rs`, manifiestos/lockfiles y `capabilities/default.json`. No edites esos puntos por deducción mientras otro colaborador los integra.

El Patient Hub es una excepción: Rust consume su API y conserva una vista local de sus resultados, sin copiar sus reglas o mantener otra autoridad. No presentes una acción del Hub como completada cuando su servicio no la aceptó.

Los últimos permisos válidos siguen en uso sin conexión; no se pueden cambiar permisos offline. La reconciliación de revocaciones exige versiones y referencias temporales verificables, no solo la fecha local.

Nightingale Patient Hub (web y API propia) y Nightingale API (Python/FastAPI, sincronización, transacciones y workers) tienen repositorios separados de este escritorio. El Hub llama a Nightingale API para consultar y modificar citas. Recordatorios, integración con n8n y analítica pertenecen a módulos y workers de Nightingale API; no se ejecutan dentro de transacciones de sincronización. Sus contratos de consumo pueden estar aquí; no añadas servidores o aplicaciones web. Sigue los [controles de planificación](tasks/README.md#architecture-checks-for-each-plan), el [ADR de tres proyectos](docs/decisions/transactional-sync-three-projects.md) y las especificaciones de [API](docs/specifications/nightingale-api.md) y [Hub](docs/specifications/nightingale-patient-hub.md).

## Antes de pedir revisión

Ejecuta desde raíz:

```sh
pnpm install --frozen-lockfile
pnpm quality
pnpm build:desktop
git diff --check
```

Prueba la ventana si el cambio afecta al arranque, la vista, IPC, permisos o persistencia. Indica sistema y arquitectura. Explica qué cambió, por qué y cómo lo comprobaste; adjunta resultados reales y separa lo pendiente. Esta base no incluye jobs de CI; ejecuta los controles locales y prueba la apertura.

Incluye los lockfiles si cambian dependencias. Revisa los archivos añadidos: no subas `.env`, credenciales, bases, registros con datos personales o recursos de pacientes. Los ejemplos deben ser ficticios. Esta base no admite datos reales hasta cerrar cifrado, claves y controles de acceso.

Si cambias un comando, requisito o ruta, actualiza la guía que seguirá el equipo. No cambies el alcance del Master Plan como efecto de una implementación.
