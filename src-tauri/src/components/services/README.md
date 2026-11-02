# Catálogo de servicios

Responsabilidad: catálogo del profesional, ofertas de la cuenta y perfil profesional.
El propietario escribe; el asistente consulta. No calcula disponibilidad ni reservas.

Datos: service_catalog_view, service_offer_view, professional_view, service_local y
service_offer_local, migración `0006_services`. Cada fila tiene ámbito de cuenta,
usuario y dispositivo. Las vistas aceptadas y las intenciones tienen distintos
escritores; remove/reset nunca borran intenciones.

API prevista en C01: list_catalog, create_service, update_service,
set_service_status, offer_service, update_service_offer, set_service_offer_status y
update_professional_profile. CatalogQuery devolverá datos aceptados a Agenda/Citas.
La UI recibirá además borradores y estados de sincronización, separados de la vista.

Cada mutación offline validará permiso y escribirá su intención con record_operation
en la misma transacción IMMEDIATE. service_local incluye intenciones de perfil sin
serviceId. Una duración admite enteros 1–1440; un ajuste null vuelve a la base.
Una oferta conserva version y serviceVersion: igual version con serviceVersion mayor
sí actualiza su vista. Desactivar servicios/ofertas no altera citas.

Estado T02: estructura y migración disponibles; casos, proyectores e IPC pendientes
de T07–T10. El contrato aprobado está en
[plan.md](../../../../tasks/P-20261102-01-account-services-catalog/plan.md).
