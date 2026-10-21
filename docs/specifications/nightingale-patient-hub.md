# Especificación técnica breve: Nightingale Patient Hub

**Versión:** 0.1
**Fecha:** 2026-10-21
**Estado:** Arquitectura acordada; proyecto separado aún no implementado aquí.
**Decisión:** [Tres proyectos y API transaccional](../decisions/transactional-sync-three-projects.md).

## 1. Propósito y partes

Nightingale Patient Hub permite gestionar una cita existente desde un teléfono, sin instalar Nightingale ni acceder al escritorio del profesional. Tendrá un repositorio propio con web adaptable y API del Hub. El backend conserva Python/FastAPI; el framework de la web queda por definir.

El navegador llama a la API del Hub. El escritorio Rust también consume sus capacidades cuando corresponde. El backend del Hub consulta citas y envía sus acciones a Nightingale API mediante HTTPS autenticado. No recibe credenciales ni acceso directo al PostgreSQL operativo.

## 2. Alcance y propiedad

| Propietario | Datos y reglas |
|---|---|
| Patient Hub | Emisión, validación, vencimiento y revocación de accesos; alcance del paciente; reglas de cancelación; solicitudes de reprogramación e intentos relevantes. |
| Nightingale API | Aceptación y persistencia compartida de citas, versiones, transiciones comunes, restricciones, historial, resultados idempotentes y Change Log. |
| Escritorio Rust | Gestión local de agenda y citas; selección de otro horario por el personal; vista derivada de resultados del Hub. |

El almacenamiento privado del Hub debe elegirse en su plan: conserva sus accesos y solicitudes, no otra agenda definitiva. La persistencia física no cambia la propiedad de las reglas ni concede acceso al esquema operativo.

El paciente puede consultar fecha, hora, profesional, servicio, lugar e instrucciones administrativas; confirmar, cancelar dentro de las reglas y solicitar reprogramación. No puede crear citas, explorar la agenda, elegir/reservar otro intervalo ni ver información ajena o clínica.

## 3. Acceso y acciones

El acceso se limita a una cita, acciones permitidas, vencimiento, revocación y política de versión. El enlace no contiene información personal legible. Se conserva solo la representación necesaria para verificar el secreto de acceso de forma segura; el formato exacto se definirá antes de implementar.

En una acción sobre la cita:

1. El Hub valida enlace o sesión, alcance y reglas del paciente; consulta el estado vigente mediante Nightingale API.
2. Envía la intención con identificador idempotente, cita, versión y contexto de actor verificable por el contrato interno.
3. Nightingale API vuelve a comprobar autorización y reglas comunes y confirma o rechaza su transacción.
4. El Hub muestra el resultado conocido. Ante un corte o timeout, consulta o reintenta la misma operación; no inventa un éxito ni crea un segundo efecto.

La cancelación aceptada libera el intervalo. Una confirmación no vale sobre una versión incompatible; reprogramar invalida la confirmación anterior según contrato. El registro de una solicitud de reprogramación pertenece al Hub y no garantiza cambio de horario. El personal la revisa; Rust propone el cambio de cita y coordina la resolución de la solicitud por la API del Hub. La relación completa entre solicitud y estado `reschedule_requested` sigue pendiente.

## 4. Disponibilidad y límites de confianza

El Hub requiere conexión con su backend y, para consultar o cambiar citas, con Nightingale API. Puede operar con los escritorios apagados. Una caída de API produce un estado de indisponibilidad o resultado pendiente de comprobar; no una confirmación local de la cita.

El escritorio conserva solo una vista recibida del Hub con versión y antigüedad. Sin conexión puede mostrarla, pero no declarar completada una acción del Hub. El resto de los casos de uso locales sigue disponible.

No hay una transacción global entre ambos proyectos. La conciliación de solicitudes y cambios de cita debe tolerar fallos parciales sin perder la intención ni aplicar dos veces el resultado.

## 5. Seguridad y privacidad

- HTTPS, respuestas mínimas y autorización de cada acceso y acción. Una identidad técnica del Hub no equivale a permiso para operar cualquier cita.
- Enlaces aleatorios, limitados y revocables; duración, reutilización y verificación adicional pendientes. Un enlace GET no ejecuta confirmaciones o cancelaciones por abrirse o previsualizarse.
- Límites de solicitudes y respuestas que no permitan enumerar citas. Definir protección contra filtración de enlaces en registros, referentes, cachés y recursos de terceros.
- Sesiones y cookies, si se usan, con controles adecuados de navegador y protección frente a acciones inducidas desde otros sitios. No entregar credenciales internas de API al navegador.
- Sin diagnósticos, notas internas, auditoría, secretos, agenda general ni datos de otros pacientes en la vista pública.
- Auditoría de acciones aceptadas y registro de intentos sin guardar tokens ni cuerpos personales innecesarios.

## 6. Contratos, comprobaciones y pendientes

Patient Hub posee su contrato público web/escritorio. Nightingale API posee el contrato interno de operaciones y consultas de citas; ambos acuerdan alcance, actor, versiones, errores, reintentos y compatibilidad. Ninguna copia del escritorio reproduce la política del Hub.

Comprobaciones futuras: enlace vencido/revocado, acceso a otra cita, versión antigua, doble envío, respuesta perdida tras aceptación, caída de API, reprogramación concurrente, solicitud rechazada/retirada y apertura del enlace por previsualización. Se probará el recorrido móvil y accesible. No hay resultados de ejecución en este repositorio.

Pendientes: framework web, almacenamiento privado, sesiones/enlaces y verificación adicional, reglas temporales de cancelación, ciclo completo de solicitudes, contrato interno, métricas, despliegue y retención. Recordatorios, n8n y analítica pertenecen a módulos y workers de Nightingale API; el Hub no los ejecuta ni decide asignaciones de lista de espera.

## Referencias

- [Master Plan](../master-plan.md), [SRS](../srs.md) y [especificación de Nightingale API](nightingale-api.md).
- [OWASP REST Security](https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html): autorización, HTTPS y validación de transiciones.

Esta es la fuente inicial hasta el traslado explícito al repositorio de Nightingale Patient Hub; aquí quedará una referencia, no otra especificación editable independiente.
