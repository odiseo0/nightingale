# P-20261108-06-appointment-creation-ui: Nueva cita con paciente existente

**Status:** Completed (2026-11-09, aceptación implícita del usuario: al solicitar continuar con el roadmap cerró la revisión C02; NOT CHECKED registrados en todo.md)
**Created:** 2026-11-08
**Updated:** 2026-11-08
**Task list:** [todo.md](todo.md)

## Overview

Construir **S04 · Nueva cita con paciente existente**, como continuación de `P-20261108-05-agenda-ui-foundation`.

El recorrido permitirá abrir Nueva cita desde Agenda, seleccionar datos reales del contexto, elegir un intervalo calculado por Rust, revisar el resumen y guardar una cita provisional en SQLite antes de volver a Agenda con su contexto conservado.

**Dependencia previa:** RESUELTA el 2026-11-08. C02 de `agenda-ui-foundation` cerró sus comprobaciones de sesión/datos con una sesión fixture real (API local + Keycloak, credenciales del usuario); las restantes (texto 200 %, lector de pantalla, corte de red físico) quedaron registradas como NOT CHECKED ambientales y el usuario, informado, dirigió continuar. **El plan está aprobado y autorizado para ejecución completa sin pausas de revisión.**

## Source of Truth

- `docs/design-master-plan.md` §§5–10.3, 16–24 y 27–31.
- `DESIGN.md`, `DESIGN-RULES.md` y `.agents/laws-of-ux/SKILL.md`.
- `docs/master-plan.md` §§4, 9.7, 13–15 y 22.
- `docs/contracts/api-v1/nightingale-api.md` §§4–5.4.
- `docs/technical/components/patients.md`: consultas de contactos y vínculos.
- `tasks/P-20261108-05-agenda-ui-foundation/`: shell, Agenda, detalle y C02 pendiente.
- `src/App.tsx`, `src/app/AppLayout.tsx`.
- `src/features/appointments/AgendaPage.tsx`, `AppointmentPage.tsx`, `AppointmentDetail.tsx` y `presentation.ts`.
- `src/shared/api/appointments.ts`: contexto y `createAppointment`.
- `src/shared/api/scheduling.ts`: evaluación local y cobertura.
- `src-tauri/src/components/patients/adapters/inbound_contacts.rs` y `application.rs`: `patients_list_patient_contacts` y `RelationshipView`.

## Architecture Decisions

### Alcance funcional y nativo

- Propietarios nativos: `appointments`, `scheduling` y `patients`.
- Casos de uso: consultar contexto, evaluar disponibilidad local, consultar vínculos del paciente y crear una cita local.
- Datos SQLite: contexto de pacientes/ofertas/profesionales, disponibilidad y versiones, relaciones responsables, cita provisional y operación Outbox.
- Interfaces existentes:
  - `appointments_get_booking_context`.
  - `scheduling_get_local_availability`.
  - `patients_list_patient_contacts`.
  - `appointments_create_appointment`.
- Solo se creará un cliente TypeScript para comandos de pacientes ya registrados en Rust. No se añadirán comandos, SQL, HTTP, migraciones ni reglas de negocio en React.
- Offline: contexto, disponibilidad, relaciones y creación usan IPC local. No se consulta disponibilidad remota ni se fuerza aceptación API.
- Sync: crear produce una operación provisional pendiente. Aceptación, descarga posterior, reintentos y conflictos compartidos se presentan solo según resultado conocido; no se simulan.
- Patient Hub y registro de pacientes quedan fuera.

### Recorrido y alcance UI

Nueva ruta: `/cita/nueva`, con entrada desde el botón **Nueva cita** de Agenda y retorno a `/agenda`.

El formulario incluirá:

1. Paciente existente, con nombre y contexto de contacto permitido.
2. Profesional y servicio/oferta real.
3. Fecha civil y selección de intervalo disponible.
4. Responsable cuando la ficha lo requiera, seleccionado mediante vínculos reales.
5. Resumen y guardado.

No incluirá registro de pacientes, creación de contactos, reprogramación, cancelación, confirmación, atención/ausencia, Semana/Lista, edición de duración personalizada ni centro de conflictos.

La duración usada en esta etapa será la **duración efectiva de la oferta**. Es una decisión acotada de este primer recorrido, no una decisión global sobre duración fija/variable.

### Selección de paciente, servicio y responsable

- Los pacientes provienen de `BookingContext.patients`; no se solicitarán IDs.
- Si no hay pacientes/profesional/ofertas utilizables, se mostrará el estado y no un formulario vacío con acciones inútiles.
- Las ofertas presentarán nombre, profesional y duración efectiva; las opciones de una sola elección podrán mostrarse como valor en contexto en lugar de selector redundante.
- Se añadirá `src/shared/api/patients.ts` para consultar `patients_list_patient_contacts` con el paciente elegido.
- Para `identificationBasis=minor_without_document`, el responsable será obligatorio y se consultarán vínculos activos marcados como responsables. Se mostrarán nombre, parentesco/función, teléfono y vigencia cuando estén disponibles; la elegibilidad final a la fecha de la cita la valida Rust.
- No se solicitará `responsibleContactId` como texto ni se crearán responsables en este plan.

### Disponibilidad y resumen

- Al elegir profesional/oferta/fecha, se llamará `getLocalAvailability` para el día civil y la duración efectiva.
- Los intervalos devueltos por Rust serán las únicas opciones de hora ofrecidas como selección rápida; no se calcularán huecos en React.
- **Decisión de implementación (verificación real):** los intervalos de Rust pueden ser *ventanas* mayores que la duración efectiva (regla de día completo). La cita se ancla al inicio del intervalo elegido con la duración efectiva de la oferta (`endsAt = min(fin de ventana, inicio + duración)`). Rust acepta subventanas y sigue siendo la autoridad que rechaza solapamientos.
- **Observación (fuera de alcance Rust):** con reglas degeneradas solapadas (0–1440 y 8–12 el mismo día), la lista de disponibilidad y la validación de creación pueden discrepar en el borde de la ventana; la UI presenta el error seguro de Rust y conserva el formulario. No se duplican reglas en React.
- La cobertura se mostrará de forma distinguible:
  - Dentro de ventana descargada.
  - Sin ventana descargada, usando solo datos locales conocidos.
  - Cobertura insuficiente: ausencia de intervalos no prueba ausencia de huecos.
- Cambiar datos que afecten la disponibilidad invalidará la selección anterior y conservará el resto del formulario.
- El resumen mostrará paciente, profesional, servicio, fecha, intervalo, duración, responsable si aplica y zona.
- Consultar disponibilidad no reserva; guardar no confirma aceptación compartida.

### Guardado, errores y retorno

- El envío usará los instantes exactos del intervalo elegido, `createAppointment` y el responsable seleccionado cuando aplique.
- Se mostrará progreso y se impedirá una segunda intención accidental.
- Se conservarán todos los campos ante errores, incluidos `SLOT_UNAVAILABLE`, `INSUFFICIENT_COVERAGE`, `VERSION_CONFLICT`, `DEPENDENCY_FAILED`, `NO_CHANGES`, `PERMISSION_DENIED`, `AUTHENTICATION_REQUIRED` y fallos temporales.
- El texto será seguro y orientado a la acción; no se interpolarán objetos de error arbitrarios.
- Tras guardar se mostrará **Guardada en este equipo · Pendiente de aceptación**, nunca Reserva confirmada.
- Al volver se conservarán fecha/filtros/selección y foco. La nueva cita se seleccionará cuando el resultado permita identificarla; en otro caso se volverá a la fecha con un mensaje persistente de provisionalidad.
- Cambios sin guardar usarán protección de navegación compatible con el router instalado, además de `beforeunload`. La solución se verificará contra documentación oficial de React Router; no se creará un mecanismo global frágil si el router ofrece bloqueo.

### Diseño y UX

- Se reutilizarán shell, tokens, fuentes, iconos, badges y estilos de la etapa anterior.
- Formulario agrupado y resumen junto al contenido cuando quepa; en una columna, resumen antes de la acción final.
- Etiquetas persistentes, errores junto al campo/resumen, foco útil, teclado completo y lectura de estados.
- Solo primitives consumidos por este recorrido; no se creará una librería completa ni un gestor global de estado.

## Dependency Order

```text
C02 de agenda-ui-foundation
  → T01: ruta y contexto del editor
  → T02: campos y responsables → C01
  → T03: disponibilidad y resumen
  → T04: guardado local y retorno → C02
```

## Task List

### Phase 1: Editor y contexto
- [x] T01: Añadir Nueva cita, ruta y carga de contexto.
- [x] T02: Implementar campos y selección de responsables.

### C01: Formulario contextual
- [x] `pnpm quality` en verde; formulario navegable y datos reales accesibles.
- [~] Revisión del usuario: autorización expresa de ejecución continua; C01 técnico.

### Phase 2: Disponibilidad y guardado
- [x] T03: Implementar intervalos disponibles y resumen.
- [x] T04: Guardar cita local, tratar errores y retornar con contexto.

### C02: Creación local completa
- [x] `pnpm quality` en verde; creación local real verificada de extremo a extremo con IPC real y sesión fixture.
- [x] Verificados: solapamiento, duplicación, reinicio, retorno, responsive (800×600/1024×768/grande), 200 %, teclado/foco.
- [ ] NOT CHECKED registrados (no bloquean el criterio técnico pero impiden declarar verificación completa): lector de pantalla, responsable de menor (sin fixture), códigos de error no disparables y corte físico de red. Revisión del usuario pendiente para cerrar el plan.

## UX laws aplicadas

| Ley | Decisión concreta |
|---|---|
| Chunking / Proximity | Paciente, servicio/horario, responsable y resumen agrupados por relación |
| Working Memory | Resumen visible y retorno con fecha/filtros/selección |
| Hick / Choice Overload | Solo opciones reales; intervalos calculados por Rust y sin duración personalizada |
| Postel / Mental Model | Controles nativos, etiquetas persistentes y errores junto al campo |
| Fitts | Acciones claras, foco visible y objetivos utilizables |
| Doherty | Progreso inmediato sin afirmar aceptación remota |

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Solicitar UUID o simular responsables | Alto | Cliente tipado para vínculos reales con nombre/teléfono/vigencia |
| Inventar disponibilidad en React | Alto | Intervalos y cobertura devueltos por Rust |
| Perder datos al cambiar contexto o fallar | Alto | Invalidar solo el slot afectado y conservar demás campos |
| Mostrar guardado local como reserva aceptada | Alto | Texto provisional persistente y resultado remoto separado |
| Crear duplicados por doble envío | Alto | Estado busy, una sola intención y verificación tras reinicio |
| Bloqueo de navegación frágil | Medio | API oficial del router y pruebas de salida atrás/adelante/descarte |
| Falta de datos de desarrollo | Alto | Preparar escenario ficticio con los flujos existentes; no declarar PASS sin evidencia |

## Open Questions

- La duración personalizada queda para una decisión posterior; esta etapa usa la duración efectiva de la oferta.
- La aceptación remota y la resolución completa de conflictos pertenecen a otros planes; aquí solo se verifica creación provisional y presentación de estados conocidos.
- Las reglas finales de elegibilidad de responsables y menores siguen en Rust; la UI no las duplica.
- Registro de pacientes/contactos desde el editor requiere un plan posterior que preserve el borrador.
