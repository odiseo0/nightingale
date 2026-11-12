# Diseño · Dispositivos y acceso (S12)

**Plan:** [../plan.md](../plan.md) · **Tarea:** T34 · **Estado:** Diseño presentado por el proceso del plan; implementación funcional bloqueada (P-20261109-15-team-devices-settings-ui).
**Fuentes:** maestro de diseño §14.5; contrato API v1 (identidad, cuenta y dispositivo); `docs/technical/components/devices.md`.

Especificación de diseño con datos sintéticos. La muestra no revoca ni autoriza dispositivos reales.

## Tarea y usuario

El dueño revisa qué equipos acceden a la cuenta y retira el acceso de los que ya no se usan, entendiendo cuándo aplica el cambio y qué resultado se conoce.

## Datos necesarios y propietario

| Dato | Propietario | Notas |
|---|---|---|
| Equipo, aplicación y último acceso conocido | Nightingale API | Metadatos permitidos; nunca tokens ni credenciales |
| Estado de autorización del dispositivo | Nightingale API | Autorizado / revocado / pendiente de verificación |
| Revocación y su resultado | Nightingale API | Se aplica al reconectar; no es instantánea offline |

## Recorrido

1. **Lista de dispositivos:** nombre del equipo, aplicación, último acceso conocido y estado. La acción principal es «Revisar acceso»; «Retirar acceso» es secundaria y siempre confirmada.
2. **Retirar acceso:** confirmación que explica el alcance: el equipo dejará de acceder al reconectar; el trabajo local ya guardado en ese equipo no se borra desde aquí.
3. **Resultado:** se muestra lo conocido («Acceso retirado»); un resultado desconocido conserva la intención y se recupera automáticamente sin avisos de proceso.

## Permisos

- Gestionar dispositivos exige el permiso de administración vigente.
- Un equipo revocado conserva los últimos permisos válidos para trabajo local ya permitido; al reconectar se comprueban revocaciones y versiones. La interfaz no promete corte instantáneo en equipos desconectados ni deduce permisos por la hora del cliente.

## Estados

Dispositivo: **Autorizado**, **Revocado**, **Último acceso antiguo** (aviso), **Verificación pendiente**. Fallos: respuesta perdida (intención conservada), rechazo (explicación y siguiente paso).

## Recuperación

- **Revocación con respuesta perdida:** no se repite la operación con otro identificador; el estado se resuelve al reconocerse el resultado.
- **Lista desactualizada:** antigüedad visible; se distingue de un fallo de consulta.

## Adaptación

Misma tabla que Equipo; a 800×600 dos filas o vacío antes que controles; nombres de equipo largos y ausencia de último acceso tienen presentación definida («No registrado»).

## Requisitos de contrato y pendientes

**Bloqueantes:** contrato remoto de dispositivos (listado, autorización, revocación, estados) y su recorrido de reconexión. Este diseño no lo inventa.

**Pendientes explícitos:** efecto exacto de la revocación sobre datos locales del equipo, verificación adicional y límites de dispositivos por cuenta.

## Muestra

Vista `#team` de [la muestra](../../../design-prototype/interface-refresh/index.html#team), bloque «Dispositivos», con equipos sintéticos y notas de diseño. Sin controles operativos.
