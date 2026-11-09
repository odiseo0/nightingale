# P-20261109-02-patient-management-ui: Registro y administración de pacientes

**Status:** Approved — en ejecución (2026-11-09; dependencias cerradas: Pacientes en lectura Completed y P06 Completed; autorización del usuario: "para este y para todos los demás puedes aprobar y ejecutar")
**Created:** 2026-11-09
**Updated:** 2026-11-09
**Task list:** [todo.md](todo.md)
**Roadmap:** [P-20261109-01-ux-ui-roadmap](../P-20261109-01-ux-ui-roadmap/plan.md) · R01

## Overview

Implementar registro y administración de pacientes, incluyendo perfiles con documento propio, menores sin documento, contactos/responsables, documentos, completitud/estado y alta desde Nueva cita conservando el borrador.

Depende de Pacientes en lectura (P-20261109-01) y P06. No comenzar hasta cerrar esas revisiones. No incluye fusión global, verificación de documentos, datos clínicos ni consentimiento no contratado.

## Source of Truth

- `docs/design-master-plan.md` §§8, 10.3, 11, 20–27 y 30–31.
- `docs/technical/components/patients.md`.
- `docs/contracts/api-v1/nightingale-api.md` §§4–5.2.
- `src/shared/api/patients.ts`, `appointments.ts`.
- Comandos `patients_register_patient`, `patients_update_patient_profile`, `patients_set_patient_completion`, `patients_set_patient_profile_status`, `patients_add_patient_identifier`, `patients_create_contact_person`, `patients_create_patient_contact_relationship`, actualizaciones/estados y consultas locales.

## Architecture Decisions

- Rust `patients` es propietario de identidad administrativa, completitud y relaciones; React solo compone formularios y resultados.
- Las mutaciones y Outbox se guardan localmente de forma provisional, sin afirmar aceptación API. Offline usa IPC; permisos/versiones/conflictos se muestran según DTO.
- Se usan comandos existentes; no se añaden tablas, SQL, HTTP ni comandos Rust.
- `draft`/`complete` y `active`/`inactive` permanecen separados. Inactivar no cancela citas; no hay Eliminar genérico.
- Documentos son texto y siempre declarados; no se verifica identidad/tutela. Teléfono código+número es indivisible. Coincidencias no fusionan personas.
- El alta desde cita preserva paciente/servicio/fecha/intervalo/responsable y foco; no sobrescribe el borrador con refrescos remotos.
- Se aplican Anek/Aileron/Phosphor, `$minimal-code`, `$docs-first-framework-code` y Laws of UX para formularios condicionales, memoria de trabajo, proximidad y errores.

## Dependency Order

```text
Pacientes lectura + P06 → T01 perfil/documento → T02 menor/responsable → C01
C01 → T03 edición/estado → T04 alta inline desde cita → C02
```

## Task List

### Phase 1: Registro administrativo
- [x] T01: Registrar perfil con documento propio e identificadores.
- [x] T02: Registrar menor con contacto/responsable y vínculo.

### C01: Altas válidas
- [x] `pnpm quality` en verde; altas locales reales y estados pendientes verificados.
- [~] Revisión del usuario: autorización continua del usuario para todos los planes.

### Phase 2: Administración y continuidad
- [x] T03: Editar perfil/documentos/estado tratando completitud y versiones.
- [x] T04: Registrar paciente desde Nueva cita conservando el borrador.

### C02: Administración completa
- [x] `pnpm quality` en verde; edición y alta inline con IPC real, errores y retorno verificados.
- [x] Accesibilidad/responsive verificados donde el entorno lo permite; NOT CHECKED registrados; aceptación final del usuario pendiente.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Romper completitud al editar relaciones | Mostrar `COMPLETENESS_CONFLICT`; no cambiar a draft silenciosamente |
| Inventar duplicados o identidad | No fusionar por coincidencias ni sugerir identidad global |
| Perder el borrador de cita | Conservar campos/selección/foco durante subrecorrido y errores |
| Confundir guardado local con aceptación | Texto provisional y operaciones separadas |

## Open Questions

- Verificación de documentos, consentimiento y fusión quedan fuera hasta contrato específico.
- Cualquier flujo no cubierto por los DTO de error se registra NOT CHECKED, sin simularlo.
