# ADR: Operación nativa, sincronización directa y servicios cloud

**Estado:** Sustituida el 2026-10-21 por [Tres proyectos y sincronización transaccional por API](transactional-sync-three-projects.md). Se conserva como historial; sus reglas de acceso directo y reparto cloud ya no están vigentes.
**Fecha:** 2026-10-21
**Fuentes:** [Plan maestro 0.2](../master-plan.md), [arquitectura técnica 0.2](../technical-architecture.md), [SRS 0.4](../srs.md).
**Sustituye:** el diseño de API central de operación/sincronización y de monorepo con servicios del PMD 0.1 y DAT 0.1, incluida la decisión resumida ADR-006. Conserva las herramientas, rutas y plataformas del [ADR de la base de escritorio](desktop-foundation.md).

## Contexto

La documentación anterior permitía trabajo offline pero asignaba la aceptación de cambios a una API central. El usuario aclaró que Rust debe ejecutar la operación del escritorio y sincronizar directamente con PostgreSQL. Python/FastAPI cubre servicios cloud de apoyo en otro repositorio.

El usuario decidió que el escritorio consuma las capacidades del Patient Hub por su API para evitar dos propietarios de las mismas reglas y resultados. También confirmó que los permisos locales permanecen sin red y que sus modificaciones necesitan conexión.

## Decisión

1. **Operación nativa.** React llama a comandos Tauri concretos. Rust ejecuta las reglas y casos de uso y guarda resultados en SQLite. Consultar o modificar datos locales permitidos no exige HTTP ni conexión activa con PostgreSQL. La agenda de hoy y mañana sigue disponible después de su descarga o creación, incluso tras reiniciar sin red.
2. **Estado compartido.** El coordinador Rust sincroniza operaciones mediante un adaptador PostgreSQL directo. No hay API central para pacientes, agenda, citas o sincronización. No se comparten archivos ni se replica indiscriminadamente todo el contenido de ambas bases. SQLite conserva intenciones locales; PostgreSQL confirma los cambios compartidos.
3. **Concurrencia e integridad.** Rust valida el estado conocido. PostgreSQL protege las escrituras compartidas con permisos, restricciones, transacciones y versiones. Una reserva offline puede entrar en conflicto con otra que el dispositivo aún no conoce. Se conservan Outbox, resultados idempotentes, Change Log, cursor e intención local ante un rechazo.
4. **Patient Hub.** El servicio es el único propietario de sus accesos, acciones y solicitudes. Rust utiliza su API y conserva solo una vista derivada, con su versión y antigüedad. No repite las reglas del Hub ni declara completada una acción sin su aceptación. El Hub puede registrar resultados con los escritorios apagados; API y sincronización deben reconciliarlos sin efectos duplicados ni sobrescrituras antiguas. El contrato separará las capacidades nativas de Citas de las del Hub.
5. **Permisos.** Sin red se usan los últimos permisos válidos y no se modifican roles, membresías o permisos. Cada operación pendiente identifica la versión de permisos usada y su fecha local. Las modificaciones remotas registran versión y fecha de efecto verificables. Al reconectar se comprueban revocaciones; la hora del equipo no prueba por sí sola que una operación fue anterior a una revocación. La política temporal de aceptación y su evidencia deben cerrarse antes de implementar identidad/sincronización. Las operaciones dudosas se conservan para rechazo o revisión.
6. **Servicios de apoyo.** Python/FastAPI, Hub web, planificación y ejecución de recordatorios, colas, reintentos, registro persistente, n8n y procesamiento analítico viven en otro repositorio. Rust registra intenciones y consume resultados cuando corresponda. Un servicio no conoce cambios que aún solo existen en un equipo sin conexión. n8n utiliza la API del servicio de recordatorios y no posee la agenda ni escribe directamente en sus tablas.
7. **Colaboración.** Cada componente tiene datos, contratos y pruebas propios. Los planes fijan los contratos compartidos antes del trabajo paralelo y verifican el recorrido sin red. La caída de un servicio de apoyo no bloquea funciones nativas ajenas a él. Las bases PostgreSQL de desarrollo serán remotas y separadas por colaborador, sin Docker; n8n de desarrollo será una instancia remota del trabajo cloud.

## Alternativas consideradas

- **API central para toda operación compartida:** era el diseño anterior; el usuario lo sustituye por coordinación nativa y acceso PostgreSQL directo.
- **Duplicar las capacidades del Hub en Rust:** produciría dos implementaciones de sus reglas y dos posibles autoridades; se consume una única API.
- **Usar fechas locales como prueba suficiente de permisos:** un reloj atrasado, adelantado o modificado no demuestra cuándo se realizó una operación. Se requieren versiones y referencias temporales verificables.
- **Cambiar permisos sin conexión:** contradice la decisión del usuario; los últimos permisos válidos se conservan hasta la reconciliación.

## Consecuencias y decisiones pendientes

- El cliente PostgreSQL tendrá acceso remoto limitado por identidad y cuenta, con cifrado. No se distribuirán credenciales de administrador ni credenciales comunes con acceso a todas las cuentas. Faltan el mecanismo de identidad, el alta de credenciales, la topología de acceso y el tratamiento de revocación.
- Antes de sincronizar se definirán el contrato transaccional, la asignación y orden de confirmación del Change Log, los reintentos, la política de permisos y la propiedad de migraciones PostgreSQL entre repositorios. No se asigna ese diseño a un servicio HTTP central.
- El contrato del Hub debe precisar acciones del paciente y del personal, versiones, invalidación de confirmaciones, intenciones pendientes y convivencia con los cambios nativos de Citas. No se deducen aquí transiciones de reprogramación que aún no están definidas.
- El alta de identidad remota, la descarga de datos remotos y los cambios de permisos requieren conexión. El arranque y el acceso local con identidad ya establecida no deben depender de una API. No se promete una revocación instantánea en un equipo desconectado ni información que nunca recibió.
- Las escrituras locales y sus cambios pendientes deben sobrevivir al reinicio. Una operación guardada localmente no implica una reserva compartida ni un recordatorio programado en la nube.
- Esta decisión cambia la documentación objetivo. No añade funciones, dependencias, credenciales o servicios a la base actual y no reabre los planes cerrados.

## Fuentes técnicas

- [PostgreSQL 18: restricciones](https://www.postgresql.org/docs/18/ddl-constraints.html): integridad de los datos compartidos.
- [PostgreSQL 18: seguridad por fila](https://www.postgresql.org/docs/18/ddl-rowsecurity.html): acceso por identidad y cuenta; no sustituirlo por un filtro que solo aplique el cliente.
- [PostgreSQL 18: autenticación de clientes](https://www.postgresql.org/docs/18/client-authentication.html): identidad y permisos de las conexiones remotas. Estos enlaces sustentan los límites; no seleccionan por sí solos el mecanismo de acceso.
