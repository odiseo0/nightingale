# Nightingale · Prototipo de diseño

Prototipo independiente de la aplicación de escritorio y del recorrido del paciente. Contiene 34 vistas navegables y una captura de cada una en `previews/`. No modifica la aplicación Tauri.

## Abrir

Abre `index.html` en un navegador. Los archivos, la fuente Inter y los iconos son locales: el prototipo no necesita paquetes ni conexión a servicios.

También puedes servir solo esta carpeta desde la raíz del repositorio:

```powershell
python -m http.server 8766 --bind 127.0.0.1 --directory design-prototype
```

Después abre `http://127.0.0.1:8766/`. Detén el servidor con Ctrl+C cuando termines. La sesión usada durante la revisión utilizó el Python incluido con el entorno de trabajo; este comando requiere tu propio Python disponible en PATH.

El enlace **Ver todas las pantallas**, al pie, abre el índice. Para empezar por él, usa `index.html#/pantallas`. La fecha del ejemplo es el jueves 22 de octubre de 2026, en Caracas.

## Recorridos

- Agenda: Día, Semana y Lista de citas; fechas, filtros y detalle.
- Citas: crear, comprobar un solapamiento, reprogramar, confirmar, registrar atención/ausencia y cancelar con confirmación.
- Pacientes: buscar, abrir ficha, registrar y editar. También se puede registrar un paciente desde el editor de cita conservando sus campos.
- Solicitudes: revisar la petición junto a la cita actual y pasar al editor de horario.
- Lista de espera: añadir una entrada, revisar horarios y ver una oferta de ejemplo.
- Comunicaciones: revisar entregas y problemas de contacto; sin chat libre ni envíos reales.
- Configuración: cuenta, servicios, horario, comunicaciones, personas y dispositivos.
- Patient Hub: consultar, confirmar, cancelar, solicitar cambio y revisar resultados/errores. Las etapas de este recorrido son ejemplos independientes del escritorio; no simulan una integración entre servidores.

Los datos solo viven en memoria. Recargar la página restablece el ejemplo. Los formularios principales protegen sus cambios al salir mediante un enlace. Los cuadros de configuración de horario, invitaciones y preferencias ilustran la decisión y su confirmación; no implementan ni conservan la configuración de una consulta real.

**Probar un escenario** cambia la presentación entre funcionamiento normal, falta de conexión, servicio no disponible y conflicto. Es un control del prototipo, fuera de la navegación del producto. En funcionamiento normal no aparecen etiquetas de sincronización en las citas ni un aviso técnico permanente.

## Índice de vistas y capturas

Cada ruta se añade después de `index.html#`. Todas las vistas están guardadas en los archivos compartidos `index.html`, `styles.css` y `app.js`; no requieren un servidor que resuelva rutas.

| Pantalla | Ruta | Captura |
|---|---|---|
| Acceso | `/acceso` | [Ver](previews/acceso.png) |
| Preparación | `/preparacion` | [Ver](previews/preparacion.png) |
| Agenda del día | `/agenda` | [Ver](previews/agenda.png) |
| Agenda semanal | `/agenda/semana` | [Ver](previews/agenda-semana.png) |
| Lista de citas | `/citas` | [Ver](previews/citas.png) |
| Detalle de cita | `/cita/a1` | [Ver](previews/cita-a1.png) |
| Nueva cita | `/cita/nueva` | [Ver](previews/cita-nueva.png) |
| Reprogramar | `/cita/a1/reprogramar` | [Ver](previews/cita-a1-reprogramar.png) |
| Pacientes | `/pacientes` | [Ver](previews/pacientes.png) |
| Ficha del paciente | `/paciente/p1` | [Ver](previews/paciente-p1.png) |
| Registrar paciente | `/pacientes/nuevo` | [Ver](previews/pacientes-nuevo.png) |
| Solicitudes | `/solicitudes` | [Ver](previews/solicitudes.png) |
| Revisar solicitud | `/solicitud/r1` | [Ver](previews/solicitud-r1.png) |
| Lista de espera | `/lista-espera` | [Ver](previews/lista-espera.png) |
| Ofertas | `/lista-espera/ofertas` | [Ver](previews/lista-espera-ofertas.png) |
| Comunicaciones | `/comunicaciones` | [Ver](previews/comunicaciones.png) |
| Indicadores | `/indicadores` | [Ver](previews/indicadores.png) |
| Cuenta | `/configuracion/cuenta` | [Ver](previews/configuracion-cuenta.png) |
| Servicios | `/configuracion/servicios` | [Ver](previews/configuracion-servicios.png) |
| Horarios | `/configuracion/horarios` | [Ver](previews/configuracion-horarios.png) |
| Política de comunicaciones | `/configuracion/comunicaciones` | [Ver](previews/configuracion-comunicaciones.png) |
| Usuarios y permisos | `/configuracion/equipo` | [Ver](previews/configuracion-equipo.png) |
| Dispositivos | `/configuracion/dispositivos` | [Ver](previews/configuracion-dispositivos.png) |
| Cambios por revisar | `/actividad` | [Ver](previews/actividad.png) |
| Conflicto de horario | `/conflicto/a3` | [Ver](previews/conflicto-a3.png) |
| Diagnóstico | `/diagnostico` | [Ver](previews/diagnostico.png) |
| Hub: cita | `/hub/cita` | [Ver](previews/hub-cita.png) |
| Hub: confirmación | `/hub/confirmada` | [Ver](previews/hub-confirmada.png) |
| Hub: cancelar | `/hub/cancelar` | [Ver](previews/hub-cancelar.png) |
| Hub: cancelada | `/hub/cancelada` | [Ver](previews/hub-cancelada.png) |
| Hub: pedir cambio | `/hub/cambiar` | [Ver](previews/hub-cambiar.png) |
| Hub: solicitud recibida | `/hub/solicitud` | [Ver](previews/hub-solicitud.png) |
| Hub: enlace vencido | `/hub/enlace-vencido` | [Ver](previews/hub-enlace-vencido.png) |
| Hub: servicio no disponible | `/hub/no-disponible` | [Ver](previews/hub-no-disponible.png) |

## Criterios visuales

- Tema claro, Inter variable local, pesos 400/510/590, fondos fríos, azul para acciones y selección.
- Tablas y separadores finos, navegación compacta y detalles en contexto. El detalle pasa a su propia superficie en una ventana estrecha.
- Texto oscuro explícito en etiquetas neutras. La confirmación combina verde oscuro con un rótulo; el color no lleva solo el significado.
- Fecha y horario anterior junto al nuevo durante la reprogramación. Las acciones que cambian la cita conservan una confirmación explícita cuando corresponde.
- Formularios con etiquetas permanentes, controles nativos, foco visible y reducción de movimiento.

La propuesta se basa en [DESIGN.md](../DESIGN.md), [DESIGN-RULES.md](../DESIGN-RULES.md), [Laws of UX](../.agents/laws-of-ux/SKILL.md) y la revisión del usuario de las referencias Attio/Linear. No copia marcas ni funciones de esos productos.

## Verificación y límites

La revisión de navegador comprobó las 34 rutas a 1280×720, 800×600 y 320×720, sin desbordamiento horizontal del documento. Las tablas y la semana pueden tener su propio desplazamiento horizontal. Se inspeccionaron visualmente Agenda, detalle, editor y Hub. `verification.json` registra la comprobación de rutas y tamaños.

Se probaron creación, solapamiento con conservación del formulario, reprogramación, aviso de cambios sin guardar, cancelación, búsqueda/registro de pacientes, petición del Hub y escenarios de desconexión/conflicto. Las pruebas verifican el prototipo, no la arquitectura distribuida. No se afirma conformidad completa de accesibilidad; faltan revisión con lector de pantalla y pruebas con usuarios reales.

No hay autenticación, SQLite, sincronización, WhatsApp ni servicios externos. No se introducen datos reales. Las reglas aún abiertas —identidad, permisos, margen de ausencia, prioridad de espera, duración de citas y conciliación de solicitudes— siguen abiertas. La muestra usa intervalos de 30 minutos solo para explorar el diseño. La ausencia requiere confirmación del personal y no se deduce de la falta de respuesta.

Referencias de los controles nativos: [diálogo modal](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement/showModal) y [campo de fecha](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/input/date). La fuente procede del [proyecto Inter](https://rsms.me/inter/) y conserva su licencia en `assets/Inter-LICENSE.txt`.
