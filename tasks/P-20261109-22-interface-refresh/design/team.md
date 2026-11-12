# Diseño · Equipo: usuarios, roles e invitaciones (S12)

**Plan:** [../plan.md](../plan.md) · **Tarea:** T34 · **Estado:** Diseño presentado por el proceso del plan; implementación funcional bloqueada (P-20261109-15-team-devices-settings-ui).
**Fuentes:** maestro de diseño §14.5; contrato API v1 (identidad, cuenta y dispositivo); `docs/technical/components/identity.md`.

Especificación de diseño con datos sintéticos. La muestra no crea usuarios, invitaciones ni accesos reales.

## Tarea y usuario

El dueño revisa quién tiene acceso a la cuenta, con qué rol, y envía o cancela invitaciones. Cada persona usa su propia cuenta; no hay cuentas compartidas.

## Datos necesarios y propietario

| Dato | Propietario | Notas |
|---|---|---|
| Miembros, roles y estado de membresía | Nightingale API | Cambios requieren conexión y autorización |
| Invitaciones (enviada, aceptada, vencida) | Nightingale API | Resultado remoto conocido; sin efecto local instantáneo |
| Permisos por miembro | Nightingale API | Se muestran como capacidades, no como códigos |

## Recorrido

1. **Lista de personas:** nombre, rol y estado (activo, invitación pendiente). La acción principal es «Invitar»; las secundarias (cambiar rol, quitar) se confirman con su consecuencia.
2. **Invitar:** correo y rol; se explica que la persona deberá autenticarse con su propia cuenta.
3. **Cambio de rol o baja:** confirmación breve con alcance («perderá acceso al reconectar»); el resultado conocido se muestra al completarse.
4. **Sin permiso:** la sección se lee como vista informativa limitada; las acciones restringidas se explican junto a su control.

## Permisos

- Gestionar equipo exige el permiso de administración vigente; la API revalida al aceptar cada cambio.
- Los cambios se aplican cuando la operación remota se conoce aceptada; offline solo se muestra la última vista conocida.

## Estados

Miembro: **Activo**, **Invitación enviada**, **Invitación vencida**, **Acceso retirado** (con resultado conocido). Fallos: respuesta perdida (intención conservada, sin repetir), rechazo de permiso (explicación y siguiente paso permitido).

## Recuperación

- **Respuesta perdida:** se conserva la intención y se impide duplicar la invitación o el cambio.
- **Rechazo:** se explica qué no pudo completarse; los datos locales no se borran.

## Adaptación

Tabla con el lenguaje común; a 800×600 se leen dos filas o el vacío antes que los controles; nombres largos y ausencia de correo tienen presentación definida.

## Requisitos de contrato y pendientes

**Bloqueantes:** contrato remoto de miembros/invitaciones (listado, creación, cambio de rol, baja, estados). El contrato API v1 cubre identidad/cuenta/dispositivo pero no el ciclo completo de equipo: este diseño no lo inventa.

**Pendientes explícitos:** roles y permisos por acción, aceptación de invitaciones, vencimiento y tratamiento local tras una baja.

## Muestra

Vista `#team` de [la muestra](../../../design-prototype/interface-refresh/index.html#team), bloque «Equipo», con personas e invitación sintéticas y notas de diseño. Sin controles operativos.
