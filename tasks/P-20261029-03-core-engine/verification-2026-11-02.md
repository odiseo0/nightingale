# Revisión de checkpoints · 2026-11-02

La verificación técnica de Core-Engine pasó en Windows con API/OIDC reales. El usuario aprobó C02–C06 y la guía T19 el 2026-11-02 con «Sí, aprobar y cerrar el plan». El plan queda completado con los límites de este informe; C01 ya estaba aprobado. [Checklist](todo.md) · [Plan aprobado](plan.md) · [Siguientes tracks](../../docs/operations/core-next-tracks.md).

## Resultado por checkpoint

| Checkpoint | Evidencia comprobada | Revisión |
|---|---|---|
| C01 | Aprobado el 2026-10-29; calidad y migraciones locales siguen correctas | Aprobado previamente |
| C02 | OIDC/PKCE con navegador externo; alta y logout con almacén nativo Windows; revocación SSE y denegación posterior; revisión de secretos | Aprobado el 2026-11-02 |
| C03 | Respuesta descartada antes de reconciliar, reapertura y GET; duplicado desde el segundo dispositivo sin nueva versión; conflicto conservado; sonda Tauri aceptada con el mismo ID y un intento | Aprobado el 2026-11-02 |
| C04 | Capturas reales desde dos perfiles vacíos, cambios incrementales y reconstrucción por cursor inválido; conflicto conservado y versión remota 2; atomicidad/retiradas cubiertas además por matrices locales | Aprobado el 2026-11-02 |
| C05 | Acceso y selección de cuenta desde Tauri; sonda offline, diagnóstico local, cierre/reapertura, reconexión, confirmación y logout desde la UI | Aprobado el 2026-11-02 |
| C06 | Calidad, build Windows, matriz real de dos perfiles, seis pruebas optativas ejecutadas y documentación actualizada | Aprobado el 2026-11-02 |

## Entorno y datos de prueba

El usuario confirmó API en `http://127.0.0.1:8000/`. `/health/live` devolvió 200/alive y `/health/ready` 200 con base y avisos disponibles. La comprobación final mantiene ese estado.

Keycloak Test usa HTTPS local, realm `nightingale-test`, cliente público `nightingale-desktop-test`, Authorization Code y S256. El usuario añadió el retorno `http://127.0.0.1/oidc/callback` y conservó el retorno fijo de las pruebas del servidor. La regla de puertos loopback consta en [Keycloak 26.8.1](https://github.com/keycloak/keycloak/blob/26.8.1/services/src/main/java/org/keycloak/protocol/oidc/utils/RedirectUtils.java#L121).

La cuenta `fixture-owner` no tenía consultas. El usuario autorizó crear **Consulta de prueba Core-Engine** mediante `nightingale-api/scripts/create_practice.py`. Antes del alta se comprobó el destino administrativo: `nightingale_local`, loopback 55432 y directorio del clúster `postgres-development`. El script terminó correctamente. No se modificaron código, configuración ni migraciones de API; no se trasladaron credenciales SQL al escritorio.

Identificadores no secretos del alta:

- Cuenta: `01a19e2d-9141-74ef-98ed-f39c88053232`.
- Usuario: `01a19e2a-9a97-71e5-abaa-ed7e7a55c3d6`.
- Profesional: `01a19e2d-9213-75cf-bce6-98fde08c1f1c`.

Las pruebas conservan sus dispositivos y pacientes ficticios en API para revisión. La matriz revocó solo su segundo dispositivo. Los intentos iniciales del arnés también crearon dispositivos de prueba antes de completar la matriz; no se borraron ni revocaron dispositivos ajenos.

## Comprobaciones finales

| Comprobación | Resultado |
|---|---|
| `pnpm quality` | Código 0: build web, fmt, Clippy con `-D warnings`, 117 unitarias, 5 de integración y un doctest correctos; 6 optativas ignoradas por defecto |
| `pnpm build:desktop` | Código 0; build Windows con recursos web incluidos en `target/debug/nightingale.exe` |
| `native_store_round_trip_rotation_and_delete` | Correcta en Windows, con una entrada ficticia única |
| `manual_oidc_login_in_external_browser` | Correcta con proveedor real y navegador externo |
| `manual_c02_activation_and_local_logout` | Correcta con almacén seguro nativo y SQLite temporal |
| `manual_t13_cycle_with_development_api` | Correcta con almacén nativo, activación, captura y ciclo completo |
| `manual_c06_two_profiles_with_development_api` | Correcta con dos SQLite y secretos en memoria, adaptadores HTTP/Rust reales |
| `manual_c05_save_offline_probe` | Correcta contra la base del escritorio y la cuenta sintética explícitas; guardado local sin HTTP |
| Contrato público activo | 46 rutas y ninguna ruta fija ausente; esquemas de configuración, registro, credencial, identidad, acceso y snapshot idénticos a la copia |
| `git diff --check` | Correcto |

La matriz real descarta una respuesta confirmada antes del commit SQLite, reabre el archivo y recupera por GET el resultado original. Repetir el mismo envoltorio desde B devuelve `duplicate`; el paciente permanece en versión 1. Una actualización aceptada lleva la versión a 2; la intención incompatible de B queda en conflicto. Reemplazar su cursor por una firma inválida provoca reconstrucción y conserva ese conflicto. Un stream SSE abierto recibe revocación y el listener Rust trata después la denegación. Los archivos temporales no contienen los tokens de esa ejecución.

La prueba refresca el contexto tras registrar el segundo dispositivo: la API puede cambiar la época de permisos. Los primeros intentos detectaron un contexto viejo en el arnés; el motor lo rechazó y se corrigió el arnés. Su limpieza también recupera un mutex envenenado para conservar el error original de una prueba fallida.

## Recorrido Tauri observado

1. Abrir el build con API configurada y certificado público de desarrollo; iniciar acceso y seleccionar la cuenta sintética.
2. Ver `Descarga completada` y cero pendientes.
3. Cortar un proxy loopback temporal, guardar una sonda con Rust y observar `Hay operaciones pendientes`, contador 1.
4. Pulsar sincronización: `Sin conexión; se conserva el trabajo local`, contador 1. El diagnóstico local responde con esquema 4 y una operación pendiente.
5. Cerrar y reabrir sin conexión: identidad y operación sobreviven; SQLite conserva integridad `ok`.
6. Reponer el proxy y sincronizar: `Descarga completada`, contador 0. SQLite conserva la operación `01a19e34-f388-738c-bd1f-5f853fc1b4fa` en `accepted`, con un intento.
7. Cerrar sesión desde Tauri: `Sin sesión`, identidad local eliminada y resultado aceptado conservado. Cerrar la instancia y el proxy de prueba; API original sigue lista.

Se guardó una copia previa del SQLite local en `target/core-checkpoint-desktop-before.sqlite`. La base previa solo tenía `schema_migrations` y esquema 1; el escritorio la migró a 4. No se restauró una copia antigua sobre el resultado del recorrido.

El proxy de prueba vive solo en `target/`, no forma parte del producto y no registra cabeceras ni cuerpos. Usa Python 3.15.1 y la biblioteca estándar: [ThreadingHTTPServer](https://docs.python.org/3.15/library/http.server.html#http.server.ThreadingHTTPServer) y [HTTPResponse.read1](https://docs.python.org/3.15/library/http.client.html#http.client.HTTPResponse.read1). Cortar/reponer ese proceso comprueba también recuperación del stream del escritorio; no se detuvo el servicio API del usuario.

## Cambios y revisión de secretos

- Se corrigió una línea de sangría que impedía pasar fmt.
- `NIGHTINGALE_DEV_CA_CERT` permite confiar en un solo PEM público en desarrollo, para HTTP, OIDC y SSE. Mantiene validación TLS de nombre, firma y fecha; release rechaza el override. No instala certificados en Windows ni configura el navegador.
- Reqwest **0.12.28**, fijado en `Cargo.lock`: [add_root_certificate](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html#method.add_root_certificate) y [from_pem_bundle](https://docs.rs/reqwest/0.12.28/reqwest/tls/struct.Certificate.html#method.from_pem_bundle). Sin nuevas dependencias, cambios de lockfile, migraciones ni ediciones del contrato generado.
- Se añadieron las dos pruebas optativas, la prueba de configuración TLS y los pasos de repetición en [desarrollo](../../docs/operations/development.md).
- Inspección de catorce registros: sin patrones de Bearer, tokens API ni contraseñas de fixtures. La base de escritorio no contiene el prefijo de tokens API. Las pruebas reales comparan además los tokens completos contra los bytes de sus SQLite temporales. Los DTO y el puerto de observabilidad conservan su lista cerrada de campos sin payload ni secretos.
- El logout retiró las credenciales locales del recorrido. El plan independiente `P-20261102-01-account-services-catalog` permanece intacto.

## Límites conservados

- macOS Keychain y Linux Secret Service siguen pendientes, según la excepción ya aprobada por el usuario. Esta ejecución acredita Windows.
- La respuesta descartada modela pérdida entre HTTP y el commit local; no fuerza un corte TCP del servidor después del commit. La matriz local sí comprueba pérdida de conexión y reintento del mismo cuerpo.
- El cursor inválido comprueba `CURSOR_EXPIRED`; no espera 90 días. El vencimiento temporal de capturas y journal, el rollback y las retiradas de vista mantienen su evidencia en las matrices locales. No se atribuye a API real una espera temporal no ejecutada.
- T19 entrega contratos y guía para Pacientes/Contactos, Cuenta/Servicios, Agenda y Citas; este trabajo no implementa esos tracks ni acredita producción.
- Solo el usuario acepta los checkpoints y el cierre del plan.
