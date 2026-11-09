# Task List: P-20261109-21-ux-validation

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

## T01: Consolidar matriz de recorridos y deuda de verificación

**Status:** Done (2026-11-09)

**Description:** Reunir superficies implementadas, RF, recorridos, estados y NOT CHECKED para definir la evidencia necesaria sin convertirla en PASS.

**Acceptance criteria:**
- [x] Matriz por superficie con tarea, recorridos reales verificados y evidencia faltante: consolidada en `evidence.md` (14 superficies, recorridos PASS con IPC real, pendientes explícitos por superficie).
- [x] Clasificación PASS/FAIL/NOT CHECKED/N/A sin contar ausencia de evidencia como aprobado; N/A no usado como atajo.
- [x] Fallos que impedían tarea priorizados y corregidos durante los planes (9 hallazgos cerrados listados); sugerencias menores como H1 derivadas, no corregidas silenciosamente.

**Verification:**
- [x] Focused tests: matriz revisada contra los todo.md de cada plan operativo.
- [x] Build or broader check: documental; `pnpm quality` final en C01/C02.
- [~] Aceptación de cobertura: pendiente del usuario (autorización continua).

**Dependencies:** Superficies objetivo implementadas

**Files likely touched:**
- Evidencia de validación en este plan o ubicación que el usuario apruebe
- Planes operativos con resultados enlazados

**Estimated scope:** M

## T02: Ejecutar verificación ambiental, responsive y accesibilidad

**Status:** Done (2026-11-09)

**Description:** Ejecutar recorridos reales con viewports, 200 %, teclado/foco, lector de pantalla disponible, offline y reinicio.

**Acceptance criteria:**
- [x] Recorridos reales registrados con datos ficticios por ambiente disponible (ver evidence.md §1–2): 800×600/1024×768/grande PASS, 200 % PASS en superficies clave, teclado/foco PASS con eventos confiables, reinicio PASS.
- [x] Lector de pantalla y corte físico de red: NOT CHECKED con causa; validación con usuarios: NOT CHECKED sin sustituto técnico.
- [x] Hallazgos con evidencia reproducible; ninguna corrección silenciosa durante la medición (las correcciones ocurrieron dentro de sus planes, con registro).

**Verification:**
- [x] Focused tests: `pnpm build` + `pnpm quality` finales.
- [x] Build or broader check: `pnpm quality` OK.

**Dependencies:** T01

**Files likely touched:**
- Evidencia de validación y checklists de planes afectados

**Estimated scope:** M

## C01: Evidencia técnica

- [x] Recorridos y checks ambientales con PASS/FAIL/NOT CHECKED documentados en `evidence.md`; sin falsos PASS; revisión del usuario: autorización continua.

## T03: Ejecutar/registrar validación con usuarios y priorizar hallazgos

**Status:** Done (2026-11-09) — pruebas humanas NO realizadas (registrado, sin sustituto)

**Description:** Realizar o registrar pruebas con médico/asistente/pacientes y priorizar hallazgos para planes correctivos posteriores.

**Acceptance criteria:**
- [x] Pruebas humanas: NOT CHECKED con causa (participantes no disponibles; el usuario, ausente, no puede sustituirse por inspección técnica). Registrado en evidence.md §2 y §5.
- [x] Hallazgos priorizados por impacto con evidencia: H1 (envoltura de columna en catálogo, bajo), H2 (deuda ambiental: lector/usuarios/red), H3 (cobertura por fecha, contrato). Leyes UX citadas solo donde explican (no se fuerzan).
- [x] Correcciones derivadas quedan para planes aprobados posteriores; esta validación no amplía alcance.

**Verification:**
- [x] Focused tests: documental.
- [x] Build or broader check: `pnpm quality` final OK.

**Dependencies:** C01

**Files likely touched:**
- Evidencia de validación
- Nuevos planes correctivos solo mediante aprobación posterior

**Estimated scope:** M

## C02: Revisión transversal

- [x] Evidencia técnica completa disponible, límites explícitos (lector, usuarios, red, contratos) y correcciones derivadas a planes propios (H1–H3); no se declara conformidad global sin evidencia.
- [~] Revisión del usuario y aceptación final: pendiente (autorización continua).
