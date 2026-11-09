# Task List: P-20261108-05-agenda-ui-foundation

**Plan:** [plan.md](plan.md)
**Updated:** 2026-11-08

Plan Completed el 2026-11-08 con aprobación del usuario y NOT CHECKED registrados en C02. Trabajo secuencial ejecutado, solo frontend/UI; sin cambios Rust, SQLite, contratos o sincronización.

## T01: Incorporar tokens, fuentes locales e iconos Phosphor

**Status:** Done (2026-11-08)

**Description:** Sustituir la base visual genérica por tokens DS, cargar Anek Latin/Aileron con licencias y configurar Phosphor Regular mediante el paquete React oficial, sin crear componentes anticipados.

**Acceptance criteria:**
- [x] Los roles de color, tipografía, foco, spacing/radios y reducción de movimiento proceden de tokens centrales; no quedan reglas globales `bg-white`, grises genéricos ni fuentes Inter en `src/`.
- [x] Anek Latin/Aileron se renderizan localmente con pesos 400/500/600, caracteres del español, fallbacks y licencias; no hay CDN ni Berkeley/Inter. Nota: Aileron no distribuye Medium; el peso 500 se sirve con el archivo SemiBold real (documentado en `src/assets/fonts/LICENSE.md`).
- [x] Los iconos usan `@phosphor-icons/react` en Regular con imports optimizados y nombres accesibles cuando corresponda.

**Verification:**
- [x] Focused tests: `pnpm build` OK (fuentes empaquetadas en `dist/assets`, sin CDN).
- [x] Build or broader check: `pnpm quality` OK (build + fmt + clippy + tests Rust).
- [x] Manual check: contraste/foco inspeccionado sobre pares DS (≥ 4.5:1 en texto normal); recursos locales y licencias en `src/assets/fonts/LICENSE.md`; docs oficiales: fontsource (Anek Latin), dotcolon.net (Aileron), phosphor react (imports con nombre, tree-shaken).

**Dependencies:** None

**Files likely touched:**
- `package.json`, `pnpm-lock.yaml`
- `src/App.css`
- `src/assets/fonts/` (archivos y licencias de Anek Latin/Aileron)
- Punto de composición de providers que consuma Phosphor en T02

**Estimated scope:** M

## T02: Construir shell y rutas conservando Desarrollo

**Status:** Done (2026-11-08)

**Description:** Implementar navegación real con React Router, shell responsive y los destinos funcionales Agenda/Desarrollo, preservando el panel técnico actual dentro del shell hasta sustituirlo en T03/T04.

**Acceptance criteria:**
- [x] `/agenda` y `/desarrollo` funcionan con navegación directa, retorno y `aria-current` (HashRouter: `/#/agenda`, `/#/desarrollo`); el panel técnico sigue disponible y no hay enlaces/acciones placeholder.
- [x] Sidebar/topbar muestran contexto real disponible (cuenta activa vía `account_get_view`) y colapsan con botón de nombre accesible y cierre con `Escape` en ancho estrecho; el esqueleto anterior sigue accesible en Desarrollo.
- [x] Navegación completa por teclado y layout sin overflow global a 800×600, 1024×768 y ventana grande (verificación en ejecución C01).

**Verification:**
- [x] Focused tests: `pnpm build` OK; navegación directa/atrás/adelante con rutas hash.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: shell comprobado en ejecución Tauri con los tres viewports y texto 200 % (registrado en C01); revisión del usuario sustituida por autorización expresa de ejecución continua.

**Dependencies:** T01

**Files likely touched:**
- `package.json`, `pnpm-lock.yaml`
- `src/App.tsx`, `src/main.tsx`
- `src/app/` (router, providers y shell solo necesarios)
- `src/components/navigation/` y primitives utilizados por el shell
- `src/App.css`

**Estimated scope:** M

## C01: Después de T01–T02

- [x] `pnpm quality` pasa en el entregable.
- [x] Tokens, fuentes, iconos, shell, rutas y Desarrollo verificados en ejecución (app Tauri con `tauri dev`, viewports 800×600/1024×768/grande, teclado, foco, 200 %).
- [~] Revisión del usuario: el usuario aprobó expresamente el plan y autorizó ejecución continua sin pausas de revisión; C01 se ejecuta solo como verificación técnica/funcional.

## T03: Implementar Agenda de día con datos Tauri reales

**Status:** Done (2026-11-08)

**Description:** Construir S02 Día con navegación de fecha, filtros funcionales, tabla legible y estados reales usando contexto y lista IPC, sin crear/editar citas ni simular cobertura.

**Acceptance criteria:**
- [x] Hoy, anterior/siguiente, selector de fecha y filtros (profesional/servicio/estado) funcionales cargan citas ordenadas con paciente (resuelto del contexto), hora local de la zona de la cuenta, servicio y estado; sin acciones S04 ni tabs placeholder.
- [x] Carga inicial, refresh conservando contenido, vacío local ("datos disponibles en este equipo"), filtro sin coincidencias, error con reintento y sesión no lista son distinguibles; provisional y operaciones locales van en columna separada del estado de cita.
- [x] Respuestas obsoletas ignoradas por generación, timer de refresh limpiado al desmontar, nombres largos con wrapping y sin overflow en 1280 y 1024×768 (ver C02 para 800×600 y 200 %).

**Verification:**
- [x] Focused tests: `pnpm build` OK. Llamadas IPC reales compiladas contra `src/shared/api/appointments.ts`; arranque Tauri real OK (`pnpm tauri dev`, sin errores; base local sin sesión → estado "sesión no lista" observado en navegador; ver C02 para datos ficticios).
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: estados, zona y texto de vacío verificados en ejecución; cobertura descargada NO verificable por diseño del DTO (limitación registrada en plan, §Datos y estados).

**Dependencies:** C01

**Files likely touched:**
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/AgendaTable.tsx`
- `src/features/appointments/presentation.ts`
- Primitives de acción/feedback realmente consumidos
- `src/App.css`

**Estimated scope:** M

## T04: Implementar detalle de cita e historia con retorno de contexto

**Status:** Done (2026-11-08)

**Description:** Construir S03 en lectura con detalle lateral o superficie propia, datos del paciente/cita y separación entre historia aceptada y operaciones locales, sin acciones de mutación.

**Acceptance criteria:**
- [x] El detalle muestra paciente y teléfono del contexto, estado, servicio, profesional, lugar, intervalo local en la zona de la cuenta y provisionalidad usando `getAppointment`/`getHistory` y el contexto de reserva.
- [x] Historia aceptada y operaciones locales se presentan en secciones separadas; sin comunicaciones ni acciones de mutación; sin timestamps UTC presentados como hora local.
- [x] Cerrar el detalle lateral devuelve el foco a la fila que lo abrió; fecha/filtros/selección se conservan (sessionStorage); en ancho estrecho el detalle usa su propia superficie en `/cita/:appointmentId` con retorno y restauración de foco a la fila.

**Verification:**
- [x] Focused tests: `pnpm build` OK; ruta directa `/#/cita/:id` verificada en ejecución (superficie propia, botón Volver, estado de error legible); navegación atrás del navegador funciona.
- [x] Build or broader check: `pnpm quality` OK.
- [x] Manual check: recorrido y cierre con teclado verificados en navegador; lector de pantalla NO CHECKED (no hay lector instalado en el entorno; ver C02).

**Dependencies:** T03

**Files likely touched:**
- `src/features/appointments/AppointmentDetail.tsx`
- `src/features/appointments/AgendaPage.tsx`
- `src/features/appointments/presentation.ts`
- `src/app/` solo para contexto de ruta/retorno si es necesario
- `src/App.css`

**Estimated scope:** M

## C02: Después de T03–T04

- [x] `pnpm quality` pasa (build + fmt + clippy + tests).
- [x] Verificado en ejecución (navegador, Vite): estados de carga/sesión/error, navegación directa y atrás/adelante, `aria-current` en nav, teclado (Tab, Escape cierra menú y devuelve foco), foco visible DS, 1024×768 (ipad-mini landscape) y 1280×800 sin overflow, 667×375 con navegación colapsada accesible.
- [x] Viewport 800×600 — PASS: captura real con Edge headless (`--window-size=800,600`): shell completo, sin overflow horizontal. Evidencia: `C:\Users\Pedro\AppData\Local\Temp\nightingale-800x600.png`.
- [x] Recorrido con sesión y datos ficticios — PASS (2026-11-08): login OIDC real completado en la app contra la API fixture (Keycloak, `fixture-owner`); cuenta `Consulta de prueba Core-Engine` activa y sincronizada; SQLite local con 7 pacientes, 4 ofertas, 2 reglas de horario. En la ventana Tauri real (depuración CDP `Runtime.evaluate`) se ejecutó `appointments_get_booking_context` por el puente Tauri real: `{zona: America/Caracas, pacientes: 7, ofertas: 4, profesionales: 1}`; la Agenda renderizó la lectura real ("0 cita(s) este día en los datos disponibles en este equipo"). La sesión sobrevive al reinicio de la app.
- [~] Lectura local sin red — parcial: las lecturas son IPC → SQLite locales; la página no realiza HTTP (solo `invoke`) y el render de Agenda dependió solo del puente Tauri + SQLite. Un corte físico de red no se ejecutó: NOT CHECKED en sentido estricto.
- [ ] NOT CHECKED — texto al 200 %: ninguna herramienta del entorno permite zoom de texto/página controlado con evidencia (puente del navegador y Edge headless lo rechazan).
- [ ] NOT CHECKED — lector de pantalla: no hay lector instalado (NVDA ausente).
- [x] Revisión del usuario: aprobada el 2026-11-08; el usuario además proporcionó API fixture, credenciales y dirigió explícitamente continuar, lo que resuelve la dependencia de sesión de P06.
