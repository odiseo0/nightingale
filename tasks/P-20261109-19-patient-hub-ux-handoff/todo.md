# Task List: P-20261109-19-patient-hub-ux-handoff

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-09

Plan Blocked: trabajo solo en el repositorio propietario del Hub; no editar aplicación web desde este repo.

## T01: Redactar especificación UX del Hub en su repositorio

**Status:** Blocked

**Description:** Documentar contenido permitido, acciones y navegación móvil para consulta/confirmación/cancelación/solicitud.

**Acceptance criteria:**
- [ ] Especificación cubre cita existente, datos administrativos y acciones sin agenda pública ni nuevas reservas.
- [ ] Cancelación explica consecuencia; solicitud conserva horario; GET no ejecuta acciones.
- [ ] Diseño desde 320 px, etiquetas, foco y objetivos táctiles definidos.

**Verification:**
- [ ] Focused tests: revisión contra contrato Hub/API.
- [ ] Build or broader check: validación de enlaces/documentos del repositorio propietario.
- [ ] Manual check: aceptación del propietario del Hub.

**Dependencies:** Repositorio y contrato propietario del Hub

**Files likely touched:**
- Documentación UX en el repositorio Patient Hub, definida por su propietario
- Ningún archivo de aplicación en este repositorio desktop

**Estimated scope:** M

## T02: Definir resultados/excepciones y continuidad semántica

**Status:** Blocked

**Description:** Especificar aceptado, unknown, estado cambiado, enlace vencido/revocado y servicio no disponible, y su relación con la vista desktop.

**Acceptance criteria:**
- [ ] Cada resultado tiene texto y siguiente paso sin exponer otra cita ni inventar éxito.
- [ ] Escritorio recibe versión/estado sin repetir efectos ni copiar reglas del Hub.
- [ ] Vencimiento/verificación quedan pendientes si no hay política.

**Verification:**
- [ ] Focused tests: revisión de todos los estados con el contrato.
- [ ] Build or broader check: `git diff --check` en el repositorio propietario.
- [ ] Manual check: revisión cruzada Hub/desktop; C01.

**Dependencies:** T01

**Files likely touched:**
- Documentación de experiencia/contrato en el repositorio Patient Hub

**Estimated scope:** M

## C01: Después de T01–T02

- [ ] Propietario acepta handoff y estados; revisión del usuario.

## T03: Validar recorridos móviles reales en el proyecto Hub

**Status:** Blocked

**Description:** Ejecutar confirmación/cancelación/solicitud y excepciones en el proyecto propietario con datos ficticios.

**Acceptance criteria:**
- [ ] Recorridos móviles usan contrato real y muestran resultados conocidos/desconocidos correctamente.
- [ ] Teclado, foco, contraste, lector de pantalla y 320 px se verifican o NOT CHECKED.
- [ ] No se modifica este repositorio desktop para simular la validación.

**Verification:**
- [ ] Focused tests: comandos reales del proyecto Hub.
- [ ] Build or broader check: quality del proyecto Hub.
- [ ] Manual check: recorrido móvil y revisión del propietario; C02.

**Dependencies:** C01

**Files likely touched:**
- Implementación y pruebas del repositorio Patient Hub, fuera de este plan desktop

**Estimated scope:** M

## C02: Después de T03

- [ ] Recorridos Hub reales y handoff completos con evidencia disponible.
- [ ] Accesibilidad/NOT CHECKED registrados; revisión del usuario.
