# Contribuir a Nightingale

Empieza por el [README](README.md), la [guía de desarrollo](docs/operations/development.md) y las partes del [Master Plan](docs/master-plan.md) que afecten a tu trabajo. Sigue también [AGENTS.md](AGENTS.md) si trabajas con un agente.

## Tomar una tarea

Acuerda con el equipo la ruta y su responsable antes de cambiar contratos compartidos. Consulta los planes en `tasks/` para evitar trabajo duplicado. Un cambio amplio necesita un plan revisado antes de implementar; usa las [reglas de nombres](tasks/README.md), conserva los planes previos y registra las comprobaciones de cada tarea. No interpretes una función descrita en la SRS como una función ya construida.

Trabaja en una rama propia. Si usas Codex, el prefijo previsto es `codex/`. Conserva los cambios ajenos del directorio de trabajo. No actualices herramientas o dependencias como parte de otro cambio sin explicar por qué hace falta.

## Colocar el código

- Interfaz y estados de vista: `src/`. Añade el cliente IPC tipado junto al caso que lo usa; no pases SQL, secretos o acceso general al sistema de archivos.
- Entrada y composición del escritorio: `src-tauri/src/lib.rs`. Mantén aquí la conexión entre Tauri y los módulos; evita acumular reglas del producto.
- Reglas del dominio: módulos Rust sin dependencias de Tauri o SQLite cuando exista un caso de uso aprobado. Prueba sus entradas y salidas.
- Acceso a datos: adaptadores Rust que sirven a ese caso. Hoy `sqlite.rs` abre y valida el esquema; no define pacientes, citas u Outbox.
- Migraciones: archivos numerados en `src-tauri/migrations/`. No edites una migración ya usada para cambiar su sentido. Añade una nueva y prueba tanto el éxito como el fallo sin pérdida de datos.
- Pruebas: junto al módulo cuando permitan leer el comportamiento con su prueba. Usa bases temporales; no escribas en la base local de un colaborador.

Crea módulos y límites cuando exista comportamiento que los necesite. El núcleo Rust opera sobre SQLite sin HTTP; su coordinador sincroniza operaciones completas mediante Nightingale API por HTTPS cuando el plan lo incluya. La API revalida y acepta el estado compartido; no entregues acceso SQL o credenciales PostgreSQL al escritorio. Define los contratos antes de repartir tareas independientes.

El Patient Hub es una excepción: Rust consume su API y conserva una vista local de sus resultados, sin copiar sus reglas o mantener otra autoridad. No presentes una acción del Hub como completada cuando su servicio no la aceptó.

Los últimos permisos válidos siguen en uso sin conexión; no se pueden cambiar permisos offline. La reconciliación de revocaciones exige versiones y referencias temporales verificables, no solo la fecha local.

Nightingale Patient Hub (web y API propia) y Nightingale API (Python/FastAPI, sincronización, transacciones y workers) tendrán repositorios separados de este escritorio. El Hub llama a Nightingale API para consultar y modificar citas. Recordatorios, integración con n8n y analítica pertenecen a módulos y workers de Nightingale API; no se ejecutan dentro de transacciones de sincronización. Sus contratos de consumo pueden estar aquí; no añadas servidores o aplicaciones web. Sigue los [controles de planificación](tasks/README.md#architecture-checks-for-each-plan), el [ADR vigente](docs/decisions/transactional-sync-three-projects.md) y las especificaciones de [API](docs/specifications/nightingale-api.md) y [Hub](docs/specifications/nightingale-patient-hub.md).

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
