# Catálogo de servicios

Responsabilidad: catálogo del profesional, ofertas de la cuenta y perfil profesional.
El propietario escribe; el asistente consulta. No calcula disponibilidad ni reservas.

Datos: service_catalog_view, service_offer_view, professional_view, service_local y
service_offer_local, migración `0006_services`. Cada fila tiene ámbito de cuenta,
usuario y dispositivo. Las vistas aceptadas y las intenciones tienen distintos
escritores; remove/reset nunca borran intenciones.

API de C01: list_catalog, create_service, update_service,
set_service_status, offer_service, update_service_offer, set_service_offer_status y
update_professional_profile. CatalogQuery devuelve datos aceptados a Agenda/Citas.
La UI recibe además borradores y estados de sincronización, separados de la vista.

Cada mutación offline valida permiso y escribe su intención con record_operation
en la misma transacción IMMEDIATE. service_local incluye intenciones de perfil sin
serviceId. Una duración admite enteros 1–1440; un ajuste null vuelve a la base.
Una oferta conserva version y serviceVersion: igual version con serviceVersion mayor
sí actualiza su vista. Desactivar servicios/ofertas no altera citas.

CatalogUseCases comprueba acceso activo para lectura y allows_offline para cada
mutación. La escritura también comprueba la autorización guardada en SQLite, su
época y que el perfil pertenezca al usuario actual. Los asistentes solo consultan.
Una oferta de un servicio aún pendiente depende de su operación CreateService.

SqliteCatalogRepository implementa CatalogQuery y conserva intenciones propias en
service_local/service_offer_local. Los estados de envío y recibos se consultan por
SqliteOperationStore.operation_result; no se duplican reglas internas de sync.
accepted conserva el borrador hasta que la vista alcance la versión del recibo.
La resolución retira entonces solo la fila provisional; conserva el Outbox y su
recibo. Conflictos y rechazos quedan para revisión. Una corrección crea un ID nuevo;
reutilizar uno con otra huella falla. Una intención aún sin resolver impide otro
cambio que necesite una versión aceptada desconocida.

ServicesProjector traduce solo service, service_offer y professional. Escribe las
vistas con la transacción recibida, sin red ni intenciones. reset borra solo las tres
vistas de su cuenta/usuario/dispositivo. Una oferta de igual versión con una versión
de catálogo mayor actualiza su vista; una versión propia menor no la reemplaza.
La duración efectiva se comprueba desde datos aceptados de la oferta, nunca desde
un borrador. Los modelos remotos permanecen dentro del adaptador.

Ocho comandos IPC services_* tienen permisos específicos, registro y DTO camelCase.
El cliente está en src/shared/api/services.ts; la UI mínima en features/services.
Permite crear, editar, activar, ofrecer y ajustar duración (vacío = null = base),
con marca provisional, estados y enlace a las revisiones del panel de sincronización.
Las preferencias locales de cuenta determinan si se muestran servicios inactivos.
La UI no calcula una duración efectiva aceptada ni concede permisos de escritura.

No se añaden eventos públicos en este track. Los recibos y páginas remotas no son
eventos de negocio. Los errores usan códigos estables y mensajes sin SQL, tokens,
payloads de transporte o huellas. No se añaden dependencias.

Pruebas: `cargo test --workspace --locked components::services`; `pnpm quality` y
`pnpm build:desktop`. Se usan SQLite temporal y datos sintéticos. Las pruebas de
contrato del bundle son T11; el registro compuesto del ciclo/SSE y la integración
HTTP completa son T12. Hasta ese registro, el ciclo normal conserva su proyector
vacío. API/OIDC reales y revisión manual del usuario no se acreditan.

Fuentes: [transacciones rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html),
[bloqueo IMMEDIATE](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior),
[UPSERT SQLite](https://www.sqlite.org/lang_upsert.html),
[campos ausentes y null en Serde 1.0.229](https://serde.rs/field-attrs.html),
[comandos Tauri 2.12.4](https://v2.tauri.app/develop/calling-rust/#passing-arguments),
[permisos](https://v2.tauri.app/security/permissions/#app-manifest),
[invoke](https://v2.tauri.app/reference/javascript/api/namespacecore/#invoke) y
[efectos React 19.3.1](https://react.dev/reference/react/useEffect#fetching-data-with-effects).

El contrato aprobado está en
[plan.md](../../../tasks/P-20261102-01-account-services-catalog/plan.md).
