# C08: Recuperación, ayuda y Hub — verificación de T18–T20

Fecha: 2026-11-12. Alcance: revisión funcional del bloque C08 del plan [P-20261111-01](../plan.md). El usuario aprobó C05–C07 el 2026-11-12 («C05, C06 y C07 Aprobados, por favor continúa con T18, T19, T20, T21») y este bloque se presenta junto a C09 por esa autorización.

Entorno: prototipo original servido en `http://127.0.0.1:8766` (`python -m http.server 8766 --bind 127.0.0.1 --directory design-prototype`), navegador integrado de Kimi Code Desktop, datos ficticios en memoria. Sintaxis: `node --check design-prototype/app.js` sin errores; `git diff --check -- design-prototype` sin advertencias de formato.

## T18: Asuntos por revisar

Recorrido verificado en navegador (ruta `#/actividad`):

1. **Enlace condicional y derivado del estado real.** La navegación muestra «Por revisar» solo con intervención pendiente y con su contador (3). Los asuntos se calculan sobre el estado del ejemplo: edición rechazada (semilla), solicitud en conciliación (r4, de T11) y revocación por confirmar (Tablet de Lucía, de T16). Con el escenario de conflicto se añade el conflicto de horario de a3.
2. **Sin ruido.** La pantalla declara que las recuperaciones automáticas y los resultados por comprobar no crean avisos, y ningún ítem de esos tipos aparece.
3. **Comparación que conserva.** El conflicto mantiene propuesta y estado compartido; no ofrece forzar ni sobrescribir.
4. **Cada problema junto al dato.** La edición rechazada muestra su aviso en Configuración → Horarios; la conciliación vive en su solicitud; la revocación, en Dispositivos. «Marcar revisado» archiva el asunto del centro y del aviso.
5. **Regresión:** los recorridos de solicitudes (T11) y dispositivos (T16) alimentan los ítems sin duplicar avisos.

Captura: `T18-por-revisar.png`. Registro estructurado en `T18-checks.json`.

## T19: Búsqueda y ayuda

Recorrido verificado en navegador:

1. **Resultados agrupados (cierra H08).** «Ana María» devuelve el grupo Pacientes (dos homónimas distinguibles por teléfono) y el grupo Citas; «09:00» devuelve citas de esa hora en varias fechas. Sin coincidencias, el mensaje habla de los datos locales, no de una ausencia global.
2. **Teclado y retorno.** Ctrl+K abre, Tab alcanza los resultados, Escape cierra; tras abrir un resultado y volver con Ctrl+K, la consulta «09:00» queda restaurada y el campo enfocado.
3. **Ayuda por tarea (cierra H09).** El diálogo explica cuatro tareas con enlaces a su pantalla; conserva el acceso al catálogo y al diagnóstico, que sigue aislado y sin secretos.

Capturas: `T19-busqueda.png`, `T19-ayuda.png`. Registro estructurado en `T19-checks.json`.

## T20: Patient Hub

Recorrido verificado en navegador (rutas `#/hub/*`):

1. **Acciones con resultados claros.** Confirmar → «Asistencia confirmada»; cancelar → «Tu cita se ha cancelado» con indicación de contactar a la consulta; pedir cambio → «Tu cita conserva su horario… no reserva otra hora», sin agenda pública.
2. **Resultado por comprobar alcanzado por flujo real.** Con el escenario «servicio no disponible», confirmar lleva a «No pudimos comprobar el resultado» con reintento honesto.
3. **Servicio no disponible separado.** Nueva pantalla `hub/servicio` («No pudimos mostrar tu cita»); el catálogo deja de mezclarla con el resultado por comprobar.
4. **Cita modificada.** Nueva pantalla `hub/cita-modificada` con el horario nuevo (vie 23 oct 09:30 – 10:00) y el anterior junto a él.
5. **320 px.** Una columna, objetivos táctiles y texto legible a 320×720 en emulación móvil; captura `T20-hub-cita-modificada-320.png`.

Registro estructurado en `T20-checks.json`.

## Límites y pendientes

- Resultados ficticios en memoria: ninguna acción del Hub contacta con un servicio real.
- Zoom al 200 % y lector de pantalla: no comprobados en este bloque.
- H01–H07 de la [auditoría heurística](../heuristics-2026-11-12/evaluacion-heuristica.md) quedan registrados sin corregir por decisión de alcance; H08–H09 se cierran con este bloque.

## Estado

- T18, T19 y T20: terminadas, criterios verificados en navegador.
- C08: pendiente de revisión del usuario, presentado junto a C09 (T21) por su autorización explícita.
