# Documentación del escritorio: origen y revisión

**Importación aportada por el usuario:** disponible al iniciar la revisión del 2026-10-23.
**Revisión documental:** 2026-10-23-r2, alineación y criterios P05/P06 aprobados por el usuario; sin implementación.
**Revisión documental:** 2026-10-29, registro de `docs/contracts/` como estado vigente de Nightingale API y retiro de `docs/database-schema.md` de este repositorio; sin implementación.
**Origen:** carpeta `docs/` del proyecto [Nightingale](https://github.com/odiseo0/nightingale). El usuario aportó estos archivos; no se ha contrastado la copia con una revisión remota. La importación no incluye un identificador Git del origen.

El usuario trasladará esta documentación al repositorio original del escritorio. Se conservan los nombres, carpetas y destinos de enlaces originales. Las rutas hacia planes del escritorio se evalúan en su repositorio; esta copia no los incluye. No se modifica aquí ese repositorio ni se crea una ubicación relativa para los archivos de API.

## Versiones de origen

| Documento | Versión o fecha declarada en el origen | Ajuste del 2026-10-23 |
|---|---|---|
| [Plan maestro](master-plan.md) | 0.3, 2026-10-21 | D06, límite de reintentos, propiedad de fuentes y criterios P05/P06 |
| [Arquitectura técnica](technical-architecture.md) | 0.3, 2026-10-21 | D06, invalidación atómica de confirmación, resultado desconocido y criterios P05/P06 |
| [SRS](srs.md) | 0.5, 2026-10-21 | Permisos vigentes al aceptar, solicitud separada y ofertas con asignación por personal |
| [ADR de tres proyectos](decisions/transactional-sync-three-projects.md) | Aceptado el 2026-10-21 | D06 y criterios P05/P06 aprobados y revisables |
| [ADR de acceso SQL directo](decisions/native-desktop-cloud-boundaries.md) | Sustituido el 2026-10-21 | Ninguno; solo historial, no instrucciones vigentes |
| [ADR de la base](decisions/desktop-foundation.md) | 2026-10-19, con notas del 2026-10-21 | Sin cambios; enlace original al plan conservado |
| [Guía de desarrollo](operations/development.md) | Sin número de versión; incluye notas del 2026-10-21 | D06; enlace original a C05 conservado |
| [Referencia a Nightingale API](specifications/nightingale-api.md) | Antes: especificación inicial 0.1, 2026-10-21 | Referencia por repositorio y documento a la fuente API, sin cuerpo duplicado ni ruta entre repositorios |
| [Especificación inicial del Hub](specifications/nightingale-patient-hub.md) | 0.1, 2026-10-21 | Ninguno; referencia hasta traslado explícito al repositorio del Hub |
| [Contrato API v1](contracts/api-v1/nightingale-api.md) | Copia de consulta exportada de `nightingale-api`, revisión `api-v1.1`, corte 2026-10-29 | Ninguno; copia fija, no fuente editable (ver sección siguiente) |

## Diseño e interfaz

- [Plan Maestro de Diseño e Interfaz](design-master-plan.md), versión 0.1, estado **Aprobado**: fuente de verdad UI/UX para superficies, recorridos, componentes, estados, accesibilidad, frontend y planes derivados. Su aprobación no autoriza por sí misma la ejecución de diseño o implementación.

## Contrato API v1 (`docs/contracts/`)

La carpeta [contracts/](contracts/) conserva la copia de consulta del contrato vigente de Nightingale API, exportada desde `nightingale-api` con `scripts/export_contracts.py` (contrato v1, revisión de modelos `api-v1.1`, head `f572f9c9b3aa4cbd52284853a91e9a66435a1dff`, corte 2026-10-29; 46 rutas, 47 operaciones HTTP, 70 esquemas y 42 acciones de escritura).

- `contracts/api-v1/openapi.json`, `public-models.schema.json`, `accepted-operations.schema.json`, `sse.schema.json`, `capabilities.json` y `manifest.json` son **artefactos generados y copia fija**: no se editan a mano en este repositorio. Una actualización del contrato se recibe copiando completa la nueva exportación desde `nightingale-api`; `manifest.json` conserva las huellas de verificación.
- `contracts/api-v1/nightingale-api.md` es la guía de consulta que explica propósito, decisiones, modelo de datos, endpoints y flujos del consumidor.
- El esquema físico PostgreSQL **no vive en este repositorio**: la fuente editable es `nightingale-api/docs/database-schema.md`, conforme al propio contrato (§5.6). El borrador local `docs/database-schema.md` fue eliminado el 2026-10-29 por decisión del usuario: el escritorio no interactúa con la base de datos directamente y consume el API solo por sus contratos HTTP.

## Documentación técnica de implementación

Las guías de módulos viven en [technical/components/](technical/components/):
[cuenta](technical/components/account.md),
[servicios](technical/components/services.md),
[identidad](technical/components/identity.md),
[dispositivos](technical/components/devices.md),
[sincronización](technical/components/sync.md),
[agenda y disponibilidad](technical/components/scheduling.md),
[citas y historia](technical/components/appointments.md),
[pacientes](technical/components/patients.md),
[plataforma](technical/components/platform.md) y
[núcleo compartido](technical/components/shared-kernel.md).
El [catálogo de datos de referencia](technical/reference-data.md) documenta las
listas de países y zonas, su origen y el acceso común desde Rust.

## Propiedad y uso

- Nightingale API mantiene su especificación en `docs/nightingale-api.md`, versión 0.2 del 2026-10-23. Su registro `docs/decisions.md` distingue criterios confirmados y detalles pendientes; esas rutas pertenecen al repositorio de API.
- La SRS, el plan, la arquitectura y los ADR del producto pertenecen al escritorio. El usuario trasladará los archivos revisados; esta revisión no acredita ese traslado ni la implementación del producto.
- El Hub posee su contrato público, accesos y solicitudes. Su especificación aquí conserva su alcance inicial hasta un traslado explícito; no se crea otra fuente contractual.
- El contrato vigente de Nightingale API se consulta en este repositorio solo a través de la copia fija de [contracts/](contracts/); cualquier cambio que interactúe con el API deberá leerla primero y no editar sus artefactos.
- P05/P06 están aprobados como criterios: solicitud separada del estado de cita e interés por oferta separado de reserva autorizada por personal. El usuario podrá cambiarlos antes o durante la implementación. Los ciclos, canales, prioridades y plazos que faltan siguen pendientes.

Antes de cambiar un contrato, corregir su fuente propietaria y renovar la referencia de consumo. No volver a añadir una especificación API independiente en `specifications/nightingale-api.md`.
