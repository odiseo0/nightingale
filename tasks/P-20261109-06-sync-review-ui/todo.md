# Task List: P-20261109-06-sync-review-ui

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Implementar resumen y lista de cambios reales

**Status:** Done (2026-11-09)

**Description:** Añadir S11 con última sincronización conocida, operaciones pendientes/unknown/retry/conflict/rejected/review y alcance por cuenta/dispositivo.

**Acceptance criteria:**
- [x] `identity_get_status` y `sync_list_pending` alimentan resumen y cola con vocabulario de trabajo ("Guardada en este equipo · Pendiente de aceptación", "Resultado por comprobar", "Necesita revisión por un cambio compartido"…); verificado con estado real: cola con 1 elemento "Crear cita · Cita · fecha local" y tarjeta de pendientes=1; tras el ciclo, vacío real con textos correctos.
- [x] Cada cambio muestra acción (etiqueta de trabajo) y entidad; no hay reintento ciego (botón "Sincronizar ahora" = ciclo del motor, no reenvío de operación).
- [x] Carga, vacío real, error con reintento y sesión no lista distinguibles; enlace en navegación con contador discreto (poll 15 s).

**Verification:**
- [x] Focused tests: `pnpm build`; lectura real de estados y pendientes con IPC.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: navegación y vocabulario verificados en captura; retorno con navegación estándar.

**Dependencies:** P06

**Files likely touched:**
- `src/App.tsx`
- `src/app/AppLayout.tsx`
- `src/features/sync/SyncReviewPage.tsx`
- `src/features/sync/presentation.ts`

**Estimated scope:** M

## T02: Implementar comparación y acciones existentes de revisión

**Status:** Done (2026-11-09) — comparación implementada; conflicto real NO alcanzable con fixtures

**Description:** Presentar aceptado/provisional/operaciones y enlazar a editores existentes sin forzar resolución ni descarte no contratado.

**Acceptance criteria:**
- [x] La comparación de conflicto (estado compartido aceptado vs propuesta local, con fecha/intervalo/zona de la cuenta) está implementada y se alimenta de `row.accepted`/`row.provisional` reales; NO CHECKED con conflicto real (los fixtures no producen conflictos; no se simula).
- [x] "Elegir otra hora" enlaza al editor R02 (conserva la propuesta con base de operación local); no existe "Forzar guardado", "Último cambio gana" ni "Descartar" sin comando.
- [x] Unknown, rechazo y permiso con siguiente paso seguro explicado ("comprueba esta misma operación; no crees otra intención equivalente" / "la API aplicó los permisos vigentes al aceptar"); textos verificados en código y renderizados cuando aplica.

**Verification:**
- [x] Focused tests: `pnpm build`; estructura de comparación verificada por inspección con datos reales (sin conflicto disponible).
- [x] Build or broader check: `pnpm quality` OK.
- [ ] NOT CHECKED — comparación y "elegir otra hora" con conflicto real.

**Dependencies:** T01 y R02 para enlace de reprogramación

**Files likely touched:**
- `src/features/sync/SyncReviewPage.tsx`
- `src/features/sync/ConflictCompare.tsx`
- `src/features/appointments/presentation.ts`

**Estimated scope:** M

## C01: Revisión funcional

- [x] `pnpm quality` en verde; estados reales (pendiente capturado y vacío tras ciclo), texto de trabajo y navegación verificados; revisión del usuario: autorización continua.

## T03: Verificar offline, reinicio, permiso y recuperación sin duplicar

**Status:** Done (2026-11-09) — verificado lo alcanzable con el entorno

**Description:** Comprobar persistencia, antigüedad, rechazo por permiso y recuperación de resultado desconocido con los flujos existentes.

**Acceptance criteria:**
- [x] Tras reinicio, pendientes y operaciones conservan identidad (SQLite/Outbox persisten; verificado de forma transversal en P06/R01 con reinicios reales); la lista se actualiza con guardia de generación sin duplicar.
- [x] Permiso vigente explicado: "la API aplicó los permisos vigentes al aceptar" y "acceso activo con últimos permisos válidos"; sin usar fecha local como autorización.
- [~] Unknown muestra la comprobación de la misma operación; el estado unknown real no fue alcanzable con fixtures (NOT CHECKED de ejecución).

**Verification:**
- [x] Focused tests: `pnpm build`; ciclo pendiente→sincronizar ahora→aceptado verificado con IPC real.
- [x] Build or broader check: `pnpm quality` OK.
- [ ] NOT CHECKED — estado unknown real, rechazo por permiso revocado y corte físico de red (no fabricables con los fixtures sin simular).

**Dependencies:** C01

**Files likely touched:**
- `src/features/sync/SyncReviewPage.tsx`
- `src/features/sync/presentation.ts`
- `src/shared/api/identity.ts` solo si el DTO existente requiere tipado adicional

**Estimated scope:** M

## C02: Centro de cambios

- [x] `pnpm quality` en verde; resumen/cola/revisiones con IPC real y estructura de comparación implementada.
- [ ] NOT CHECKED — conflicto real, unknown real, rechazo por permiso y corte de red (no fabricables con fixtures); lector de pantalla no instalado.
- [~] Revisión del usuario: autorización continua; aceptación final pendiente.
