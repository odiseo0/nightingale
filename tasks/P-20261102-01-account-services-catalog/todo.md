# P-20261102-01-account-services-catalog — Tareas

**Plan:** [plan.md](plan.md)
**Status:** Draft — pendiente de aprobación del usuario

## T01: Cerrar los contratos del track

- [ ] DTO por comando (camelCase) para los 11 casos de uso (8 catálogo + 3 cuenta + preferencias).
- [ ] Tabla de acciones congelada: acción, entityType, clave (UUID/compuesta), versión esperada, online/offline (la del plan §«Contrato de sincronización»).
- [ ] Propiedad de tablas: `account_view`, `account_preferences` (migración `0005`); `service_catalog_view`, `service_offer_view`, `professional_view`, `service_local`, `service_offer_local` (migración `0006`). Números reservados aquí antes de cualquier otro trabajo paralelo.
- [ ] Firmas de los puertos públicos `AccountViewQuery` y `CatalogQuery` (consumo futuro de `scheduling` y `appointments`).
- [ ] Puntos calientes de integración asignados: `lib.rs`, `build.rs`, `capabilities/default.json`, `components/mod.rs`, registro del proyector compuesto en `identity/desktop.rs` (líneas ~434 y ~538 sustituyen `&()`).

## C01: Contratos congelados

- [ ] Revisión del usuario de DTO, acciones, tablas, puertos y puntos de integración.
- [ ] Sin implementación hasta esta aprobación.

## T02: Esqueletos de componentes y migraciones

- [ ] Crear `components/account/{domain,application,adapters/{inbound,outbound}}` + `mod.rs` y README breve (responsabilidad, datos, API, puertos, invariantes).
- [ ] Crear `components/services/{domain,application,adapters/{inbound,outbound}}` + `mod.rs` y README breve.
- [ ] `src-tauri/migrations/0005_account.sql` con `PRAGMA user_version = 5` y `schema_migrations`.
- [ ] `src-tauri/migrations/0006_services.sql` con `PRAGMA user_version = 6` y `schema_migrations`.
- [ ] Verificar apertura de base nueva y migración desde esquema 4 con pruebas del runner.
- [ ] Registrar ambos módulos en `components/mod.rs`.

## T03: Dominio y aplicación de `account`

- [ ] Entidades/objetos de valor: `AccountView`, `AccountPolicy`, `AccountDetailsPatch`, `PreferenceKey`; validaciones (patch no vacío, `noShowGraceMinutes` 0–1440, zona IANA, país ISO alfa-2).
- [ ] Casos `get_account_view`, `get_preferences`, `set_preference` (transacción local, sin Outbox).
- [ ] Política: preferencia local no puede fijar política central obligatoria.
- [ ] Pruebas de dominio sin base ni red; pruebas de aplicación con puertos falsos.

## T04: Repositorios y proyector de `account`

- [ ] `AccountProjectionRepository` (SQLite) para `account_view` escritura por proyector.
- [ ] `AccountReadRepository`: lectura de vista + preferencias fusionadas.
- [ ] `AccountProjector` implementa `RemoteProjector` para `EntityType::Account`; `reset` retira solo su ámbito; ignora otros tipos.
- [ ] Pruebas: proyección idempotente, `remove` retira vista, `reset` por captura, aislamiento por cuenta.

## T05: Casos online-only de `account`

- [ ] `update_account_details`, `update_account_policy`, `set_account_status`: exigen sesión + acceso `active` + rol propietario; refresco `/access` como prueba de conectividad; rechazo sin persistir si falla.
- [ ] Envío directo con `SyncAPI::send_operations` y envelope en memoria (ID estable por invocación entre reintentos de transporte); tratamiento de `accepted` (disparar `sync_now`), `duplicate` (`originalResult`), `conflict`/`rejected`, y resultado desconocido.
- [ ] Nada escrito en SQLite; documentar la semántica de repetición manual en README.
- [ ] Pruebas con servidor HTTP sintético loopback: éxito, 401/403 (denegación local), timeout con reintento mismo ID, rechazo visible.

## T06: IPC y UI mínima de `account`

- [ ] Comandos: `account_get_view`, `account_get_preferences`, `account_set_preference`, `account_update_details`, `account_update_policy`, `account_set_status`; permisos en `capabilities/default.json` y `build.rs`; registro en `lib.rs`.
- [ ] `src/shared/api/account.ts` como única frontera TS; DTO sin tokens ni estado remoto crudo.
- [ ] `features/settings`: vista, formularios (propietario), aviso «requiere conexión», preferencias.
- [ ] Pruebas de permisos/redacción IPC (patrón T14/T15 del motor).

## C02: Revisión vertical `account`

- [ ] El usuario revisa el recorrido completo con datos sintéticos (con y sin conexión simulada).

## T07: Dominio y aplicación de `services`

- [ ] Objeto de valor `DurationMinutes` (entero 1–1440); `ServicePatch`/`ServiceOfferPatch` no vacíos; `UpdateServiceOffer` exige `durationMinutesOverride` presente (admite `null`).
- [ ] 8 casos de uso con validación de acción vía `AccountAccess::allows_offline`; clave compuesta `{professionalId, serviceId}` validada como UUIDv7 canónicos.
- [ ] Cálculo de duración efectiva = override ?? default solo desde estado aceptado.
- [ ] Pruebas de dominio: rangos, patches, ausencia vs nulo, claves compuestas, edición de intención local previa (nueva versión de la intención, mismo ID de operación solo si el contenido no cambió — huella).

## T08: Repositorios e intenciones locales de `services`

- [ ] Escritura de `service_local`/`service_offer_local` + `record_operation` en la **misma** transacción `IMMEDIATE` (rollback conjunto).
- [ ] Resolución de intenciones desde recibos (vía consulta de resultados por `operation_id`): `accepted` marca y retira provisional al confirmar vista; `conflict`/`rejected` quedan para revisión.
- [ ] Lectura fusionada aceptada + provisionales con marca de estado de sincronización.
- [ ] Pruebas: sin red, reapertura conserva, rollback, resolución por recibo.

## T09: Proyectores de `services`

- [ ] `ServicesProjector` para `Service`, `ServiceOffer`, `Professional` (misma estructura, ámbito compartido); ignora otros tipos.
- [ ] Regla de versiones dobles: `version` igual con `serviceVersion` mayor sí se proyecta; `version` menor no.
- [ ] `reset` retira solo sus tres vistas; `remove` retira vista sin tocar intenciones.
- [ ] Pruebas: idempotencia, grupos repetidos, remove, reset de captura, regla de versiones.

## T10: IPC y UI mínima de `services`

- [ ] Comandos: `services_list_catalog`, `services_create_service`, `services_update_service`, `services_set_service_status`, `services_offer_service`, `services_update_service_offer`, `services_set_service_offer_status`, `services_update_professional_profile`; permisos, `build.rs`, `lib.rs`.
- [ ] `src/shared/api/services.ts`; UI `features/services` con marca de provisional y estados de sincronización; solo lectura para asistentes.
- [ ] Revisión de conflictos enlazada al `ReviewSummary` existente.
- [ ] Pruebas de permisos/redacción IPC.

## C03: Revisión vertical `services`

- [ ] El usuario revisa: alta offline → sincronización → aceptado; conflicto visible; asistente sin escritura.

## T11: Pruebas de contrato del consumidor

- [ ] Fixtures contra el bundle fijo: hashes del manifiesto, presencia de las 10 acciones, correspondencia acción↔modelo según `capabilities.json`, claves compuestas.
- [ ] Los modelos de estado usados ya existen en `remote_models.rs`; verificar que no hay deriva respecto al bundle.
- [ ] Nunca editar artefactos de `docs/contracts/api-v1/`.

## T12: Integración y punta a punta

- [ ] Proyector compuesto `(AccountProjector, ServicesProjector)` registrado en `identity/desktop.rs` (sustituye `&()` en ciclo y SSE).
- [ ] Escenarios sintéticos con servidor loopback y dos perfiles donde aplique: crear servicio offline → dispatch → `accepted` → proyección; `VERSION_CONFLICT` conserva intención; `remove` retira vista; reapertura conserva; oferta con `serviceVersion` mayor proyecta; captura nueva reconstruye vistas conservando intenciones.
- [ ] Confirmar que casos online-only no dejan intenciones en Outbox.

## T13: Calidad, guías y cierre

- [ ] READMEs finales de `account` y `services` (PMD §26/§27.2).
- [ ] `pnpm quality`, `pnpm build:desktop`, `git diff --check` en verde.
- [ ] Actualizar [core-next-tracks.md](../../docs/operations/core-next-tracks.md) si el recorrido real difiere de la referencia (solo con aprobación del usuario).

## C04: Aceptación del track

- [ ] Revisión del usuario de la evidencia local y decisión de aprobación.
- [ ] Pendientes externos (API/OIDC de desarrollo) quedan explícitos, no acreditados.
