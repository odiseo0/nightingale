# Evidencia de validación transversal (R20)

**Fecha:** 2026-11-09 · **Alcance:** superficies implementadas R01–R17 + P05/P06.
**Clasificación:** PASS/FAIL/NOT CHECKED/N/A. N/A y NOT CHECKED **no** cuentan como aprobados.
**Ambiente:** app Tauri real (Windows), sesión fixture (API local + Keycloak), verificación por CDP con IPC real. Lector de pantalla: **no instalado** (NVDA ausente). Corte físico de red: **no ejecutado** (la API es servicio del usuario). Pruebas con médico/asistente/pacientes: **no realizadas** (usuario ausente; no se sustituyen por inspección).

## 1. Matriz de superficies y recorridos

| Superficie (ruta) | Tarea principal | Recorrido real verificado (IPC) | Pendiente |
|---|---|---|---|
| Agenda Día `/agenda` | Qué citas hay hoy | PASS: día/filtros/detalle lateral/estados/refresh/reinicio | — |
| Agenda Semana `/agenda` (tab) | Comparar ocupación | PASS: periodo, eventos, bloqueos, scroll localizado | — |
| Agenda Lista `/agenda` (tab) | Buscar próximas/pasadas | PASS: periodo, filtros, paginación, detalle | — |
| Detalle S03 `/cita/:id` | Revisar y actuar | PASS: datos, historia/operaciones separadas, acciones del ciclo, reprogramar | — |
| Nueva cita `/cita/nueva` | Crear cita local | PASS: contexto, disponibilidad, guardado provisional, doble clic, retorno | responsable de menor (sin fixture) NOT CHECKED |
| Pacientes `/pacientes` | Directorio y búsqueda | PASS: búsqueda, filtros, paginación, retorno con foco | — |
| Ficha `/paciente/:id` | Datos administrativos | PASS: perfil, documentos, responsables, citas paginadas | — |
| Registro `/pacientes/nueva` | Alta propia/menor | PASS: ambas altas reales, provisionalidad, edición, estado, alta inline desde cita con borrador | COMPLETENESS_CONFLICT real NOT CHECKED |
| Reprogramar `/cita/:id/reprogramar` | Cambiar horario | PASS: comparación, duración conservada, invalidación de confirmación, SLOT_UNAVAILABLE | conflicto real NOT CHECKED |
| Ciclo de cita (S03) | Confirmar/cancelar/atender/ausencia | PASS: fricción, momento/margen, terminales | rechazo por permiso real NOT CHECKED |
| Cambios `/cambios` | Revisar sincronización | PASS: cola, resumen, vocabulario, sincronizar ahora; comparación implementada | conflicto/unknown real NOT CHECKED |
| Acceso `/acceso` | Entrar y preparar | PASS: rama activa + preparación real; OIDC interactivo (evidencia P05 con usuario) | cierre/re-login autónomo NOT CHECKED |
| Configuración `/configuracion/*` | Cuenta, servicios, horarios | PASS: mutaciones reales (política, catálogo, ofertas, reglas, excepciones, bloqueos), versiones | conflictos de versión reales NOT CHECKED |
| Búsqueda global (Ctrl+K) | Encontrar paciente/cita | PASS: resultados reales, apertura, Escape con retorno de foco | — |
| Diagnóstico `/diagnostico` | Metadatos locales | PASS: datos reales, sin secretos | estado sin sesión NOT CHECKED |

## 2. Ambiente transversal

| Check | Resultado | Evidencia |
|---|---|---|
| 800×600 | PASS | Edge headless + CDP `Emulation` en agenda, editor, directorio, ficha, registro, cambios, diagnóstico (scrollW < innerW) |
| 1024×768 | PASS | CDP en editor y agenda |
| Ventana grande | PASS | Capturas 1280+ en cada superficie |
| Texto 200 % | PASS | Zoom por estilo en agenda y editor (1265 < 1280); resto con mismas reglas CSS |
| Teclado/foco | PASS | Tab confiable (CDP) en editor y shell; Escape/flechas en paleta; retorno de foco en menú, detalle y paleta |
| Reinicio/offline local | PASS (reinicio) / NOT CHECKED (corte de red) | Persistencia verificada en P05/R01/R06; corte no ejecutado |
| Lector de pantalla | NOT CHECKED | Sin lector instalado en el entorno |
| Validación con usuarios | NOT CHECKED | Participantes no disponibles; registrado como pendiente explícito |

## 3. Hallazgos corregidos durante el desarrollo (cerrados)

1. Medianoche civil con offsets de zona negativos (P05→presentation.ts; el panel técnico heredado `AgendaPanel.tsx` conserva el código viejo a propósito: es diagnóstico, no producto).
2. Formato de medianoche en intervalos (00:00 inicio / 24:00 fin, `formatTimeEnd`).
3. Doble intención por doble clic y carrera blocker/navegación (P06; refs + `useBlocker` en forma función).
4. Persistencia vs restauración en modo estricto (inicialización perezosa) e invalidación de slot que borraba el borrador restaurado (P06 T04).
5. Rango máximo de 366 días del contrato local (ficha de paciente y búsqueda global).
6. Filas solo-provisionales ocultas en catálogo (R08) y estado obsoleto tras aceptación (poll 10 s).
7. Semántica de ámbito de bloqueo (`accountId` ausente/null = global; R09).
8. Mensaje de resultado sobrescrito por estado obsoleta y encadenamiento de versiones (R07).
9. Selects nativos desbordando con opciones largas al 200 % (P05/R04).

## 4. Hallazgos abiertos (para planes correctivos futuros, requieren aprobación)

- **H1 (bajo):** en el catálogo de servicios, nombres largos se envuelen muy estrechos en la primera columna a 800×600 (la tabla tiene scroll localizado; mejorable con ancho mínimo de columna).
- **H2 (deuda ambiental acumulada):** lector de pantalla, validación con usuarios y corte de red físico quedan pendientes hasta disponer de entorno/participantes (R20 §2).
- **H3 (contrato):** cobertura descargada por fecha no distingue “no descargado” de “sin citas” (limitación de DTO registrada desde P05; requiere fuente contractual, no UI).

## 5. Conclusión

Evidencia técnica completa para las superficies implementadas con los ambientes disponibles. **No se declara conformidad de accesibilidad global** (lector y usuarios pendientes). Las correcciones H1+ requieren sus propios planes aprobados; no se amplía esta validación.
