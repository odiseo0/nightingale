# Documentación del escritorio: origen y revisión

**Importación aportada por el usuario:** disponible al iniciar la revisión del 2026-10-23.
**Revisión documental:** 2026-10-23-r2, alineación y criterios P05/P06 aprobados por el usuario; sin implementación.
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

## Propiedad y uso

- Nightingale API mantiene su especificación en `docs/nightingale-api.md`, versión 0.2 del 2026-10-23. Su registro `docs/decisions.md` distingue criterios confirmados y detalles pendientes; esas rutas pertenecen al repositorio de API.
- La SRS, el plan, la arquitectura y los ADR del producto pertenecen al escritorio. El usuario trasladará los archivos revisados; esta revisión no acredita ese traslado ni la implementación del producto.
- El Hub posee su contrato público, accesos y solicitudes. Su especificación aquí conserva su alcance inicial hasta un traslado explícito; no se crea otra fuente contractual.
- P05/P06 están aprobados como criterios: solicitud separada del estado de cita e interés por oferta separado de reserva autorizada por personal. El usuario podrá cambiarlos antes o durante la implementación. Los ciclos, canales, prioridades y plazos que faltan siguen pendientes.

Antes de cambiar un contrato, corregir su fuente propietaria y renovar la referencia de consumo. No volver a añadir una especificación API independiente en `specifications/nightingale-api.md`.
