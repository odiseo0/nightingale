# Referencia: Nightingale API

**Estado:** referencia de consulta; no es una especificación editable independiente.
**Actualización documental:** 2026-10-23.
**Decisión:** [Tres proyectos y API transaccional](../decisions/transactional-sync-three-projects.md).

La fuente vigente es `docs/nightingale-api.md`, versión 0.2 del 2026-10-23, en el repositorio de **Nightingale API**. Allí también se mantienen `docs/decisions.md`, `docs/contracts.md` y `docs/access-design.md`. Esas rutas identifican documentos del repositorio de API; no son rutas relativas del escritorio ni requieren que ambos repositorios estén juntos.

Esta ruta contenía la especificación inicial 0.1 del 2026-10-21, trasladada por el usuario al repositorio de API ese día. Se conserva el archivo y su ubicación para los enlaces del escritorio, sin mantener otro cuerpo del contrato ni inventar un enlace entre repositorios. La versión indicada identifica la fuente consultada, no congela sus revisiones futuras.

D06 confirma OIDC externo y permisos vigentes al aceptar. P05/P06, aprobados por el usuario el 2026-10-23, separan solicitud y estado de cita, y respuesta a oferta y reserva autorizada por personal. El usuario podrá revisar estos dos criterios antes o durante la implementación. El ADR de tres proyectos del escritorio recoge los criterios y los pendientes; esta referencia no acredita implementación.

## Referencias

- [Master Plan](../master-plan.md), [arquitectura](../technical-architecture.md) y [especificación del Hub](nightingale-patient-hub.md).
- [OWASP REST Security](https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html), [PostgreSQL 18: transacciones](https://www.postgresql.org/docs/18/tutorial-transactions.html), [RFC 8252](https://www.rfc-editor.org/rfc/rfc8252.html) y [RFC 9700](https://www.rfc-editor.org/rfc/rfc9700.html).
