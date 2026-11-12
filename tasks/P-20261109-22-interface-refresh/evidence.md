# Evidencia consolidada — P-20261109-22 Renovación de la interfaz

**Fecha:** 2026-11-09
**Estado:** renovación de producto completa en código (T01–T29); diseños futuros entregados (T30–T36); verificación técnica automatizada PASS; verificación humana y ambiental pendiente (NOT CHECKED).

## Checkpoints técnicos

| Checkpoint | Build/quality | Pruebas de presentación | Evidencia |
|---|---|---|---|
| C01–C03 (reglas, muestra, base) | PASS (histórico) | — | evidence-c01/c02/c03.md |
| C04 (navegación, T08–T10) | quality PASS | — | evidence-c04.md |
| C05 (agenda visual, T11–T13) | quality PASS | 11/11 al cierre de fase | evidence-c05.md |
| C06 (consulta y creación, T14–T16) | quality PASS | 11/11 | evidence-c06.md |
| C07 (citas y pacientes, T17–T19) | quality PASS | 11/11 | evidence-c07.md |
| C08 (pacientes, T20–T22) | quality PASS | 11/11 | evidence-c08.md |
| C09 (configuración, T23–T26) | quality PASS | 11/11 | evidence-c09.md |
| C10 (recuperación, T26–T28) | quality PASS | 16/16 | evidence-c10.md |
| C11–C13 (diseño futuro + ayuda) | diff --check PASS | 16/16 | evidence-c11-c13.md + T29 |

Última ejecución conocida: `pnpm quality` PASS (377 unitarias, 5 persistencia, 1 doc-test; 9 ignoradas); `node --test tests/ui/*.test.mjs` 16/16.

## Qué cambió (resumen por regla aprobada)

- **Sincronización invisible:** retiradas columna/etiquetas/contadores/fases/botón de sincronizar de Día, Lista, Semana, detalle, ficha, directorio, servicios, horarios y Por revisar; textos de guardado limitados al resultado local conocido («Cita guardada.», «Cambios guardados.», «Paciente guardado.»).
- **Por revisar:** solo conflicto/rechazo/revisión requerida; enlaces por clave de entidad (sin barrido); unknown automático; sin acciones de forzar/descartar.
- **Profesional:** columna y filtro condicionales (dueño con un único profesional) vía viewerRole real; nunca inferido por canManage.
- **Direcciones:** lectura terminada en ciudad y estado (formatAddressLine); país conservado en edición y en documentos.
- **Tipografía:** Montserrat títulos / Mulish interfaz, locales; Anek retirada.
- **Semana:** 96 px/hora, 160 px/día, solapamientos en columnas, rango visible con eventos reales.
- **Formularios:** grupos Paciente/Servicio/Horario; resumen previo; línea base separa precarga de edición; anti-duplicación en pasos parciales; intención única ante doble clic.
- **Controles:** modal común `<dialog>`; búsqueda con teclado completo; shell 216/52 px con utilidades.

## Verificaciones pendientes (no cuentan como aprobadas)

1. Recorridos con IPC real e interacción humana por superficie (ratón, teclado, foco).
2. Matriz visual 800×600 / 1024×768 / 1280×720 / 1440×900 y zoom 200 % real.
3. Lector de pantalla y validación con médico/asistente.
4. macOS/Linux; corte de red real y reinicio con respuesta perdida (escenarios 10–12 del plan).
5. Aceptación expresa del usuario para C04–C14 y cierre del plan.

## Entregables

- Plan/todo actualizados; evidencias por checkpoint en esta carpeta.
- Matriz de accesibilidad: [accessibility-matrix.md](accessibility-matrix.md).
- Comprobaciones de integración: [integration-checks.md](integration-checks.md).
- Diseños futuros con dependencias: `design/` y `design-prototype/interface-refresh/`.
