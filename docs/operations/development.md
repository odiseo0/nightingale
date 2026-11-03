# Desarrollo del escritorio

Esta guía cubre la base local y el motor transversal de Nightingale. Los casos locales no requieren servicios web, PostgreSQL, n8n, Docker ni configuración privada; el acceso y la sincronización remota sí requieren Nightingale API y OIDC de desarrollo. SQLite conserva esquema, identidad/acceso, Outbox/Inbox, cursor y vista remota aceptada. Pacientes, Servicios, Agenda y Citas siguen pendientes como funcionalidades de producto.

**Revisión documental del 2026-10-23:** alineación de permisos, sin cambiar las rutas del escritorio ni repetir sus comprobaciones de ejecución.

## Límites de esta base y de las integraciones futuras

La [arquitectura vigente](../decisions/transactional-sync-three-projects.md) asigna la operación local a Rust/SQLite y la aceptación remota a Nightingale API por HTTPS. El escritorio no tiene acceso SQL o credenciales PostgreSQL. Nightingale Patient Hub tiene web y API propias y usa Nightingale API para consultar y modificar citas. Cada proyecto tiene su propio repositorio.

El escritorio implementa identidad OIDC/PKCE, activación de instalación, Outbox/Inbox, sincronización, capturas, avisos SSE e IPC de acceso/estado. No implementa el cliente del Hub ni las funcionalidades de negocio. Para las pruebas remotas, cada colaborador usará un entorno API aislado con su base remota de desarrollo; las credenciales PostgreSQL pertenecen al servidor API y a la administración autorizada, no al cliente de escritorio. No se exige Docker. Recordatorios, integración con n8n y analítica pertenecen a módulos/workers de Nightingale API. La instancia n8n de desarrollo seguirá siendo remota y su trabajo no bloqueará transacciones de sincronización.

No cambies permisos sin conexión ni aceptes una operación pendiente por su fecha local. D06, del repositorio de API, confirma OIDC externo y permisos vigentes al aceptar: si falta permiso, API rechaza sin efecto y el escritorio conserva la intención para revisión. El contrato v1 fija el flujo implementado de dispositivos; los detalles de proveedor, alta y despliegue pertenecen al repositorio de API.

## Motor transversal y colaboración

El plan `P-20261029-03-core-engine` y su [ADR](../decisions/desktop-core-engine.md) definen los componentes `identity`, `devices`, `platform` y `sync`, la propiedad de sus tablas y las migraciones `0003`/`0004`. T01–T19 tienen implementación y verificación local. El 2026-11-02 pasaron las pruebas API/OIDC reales y el recorrido Tauri en Windows; el usuario aprobó C02–C06, T19 y el cierre del plan con los límites documentados. La [evidencia de checkpoints](../../tasks/P-20261029-03-core-engine/verification-2026-11-02.md) separa resultados, entorno y límites.

Para el trabajo remoto del motor, `NIGHTINGALE_API_BASE_URL` selecciona explícitamente la API de desarrollo. No existe un destino remoto por defecto. El emisor, cliente público y audiencia OIDC se obtienen de `/v1/auth/config`; no se duplican en variables ni se guardan credenciales en `.env`. Release exige HTTPS y solo desarrollo/pruebas admiten HTTP loopback.

Cada componente modifica únicamente sus tablas y expone puertos para los demás. Antes de abrir tracks paralelos, fija en el plan DTO, acciones, proyectores, comandos, propietarios y migraciones. El integrador registra composición y permisos en los puntos calientes enumerados en `CONTRIBUTING.md`. Los artefactos de `docs/contracts/api-v1/` se sustituyen como una exportación completa del repositorio de API, nunca mediante ediciones locales. Los [contratos iniciales de los siguientes tracks](core-next-tracks.md) explican el recorrido para añadir un caso sin editar kernel ni tablas ajenas.

### Configuración y credenciales del motor (T04)

`platform::config::APIConfig::from_environment` lee únicamente `NIGHTINGALE_API_BASE_URL`, cuando un caso remoto necesita la configuración. No carga `.env`, no tiene URL por defecto y no exige esa variable para abrir SQLite o ejecutar los casos locales. La configuración rechaza usuario/contraseña, query, fragmento y protocolos distintos de HTTPS. Solo los builds de desarrollo admiten HTTP en `localhost` o una IP loopback. Versión del cliente y plataforma proceden del build y del SO; el descubrimiento OIDC se realiza durante el acceso.

Ejemplo ficticio para una API local ya iniciada, en la misma terminal del escritorio:

```powershell
$env:NIGHTINGALE_API_BASE_URL = 'http://127.0.0.1:8000/'
```

Si el proveedor OIDC de desarrollo usa un certificado propio, indica su PEM público antes de iniciar el escritorio o las pruebas:

```powershell
$env:NIGHTINGALE_DEV_CA_CERT = '<ruta absoluta al certificado público PEM>'
```

El transporte añade ese certificado a su confianza TLS para HTTP, OIDC y SSE. Conserva las comprobaciones de nombre, firma y vigencia. Acepta un solo certificado; un archivo ausente o inválido impide crear el transporte con un error sin rutas. Esta opción solo funciona con `debug_assertions`; release la rechaza. No instala certificados en Windows ni configura el navegador externo. No necesita claves privadas. Fuentes de reqwest 0.12.28: [add_root_certificate](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html#method.add_root_certificate) y [from_pem_bundle](https://docs.rs/reqwest/0.12.28/reqwest/tls/struct.Certificate.html#method.from_pem_bundle).

`platform::secrets::SecretStore` es el único puerto para credenciales. `NativeSecretStore` usa `keyring 4.2.0` con `v1`: Windows Credential Manager, macOS Keychain o Secret Service en Linux. La clave separa URL de API, perfil local estable, pareja OIDC `issuer`/`subject`, cuenta y dispositivo cuando existen, y tipo de token. El llamante debe conservar el mismo perfil entre reinicios. OIDC puede guardar su sesión antes de que API asigne el dispositivo; la credencial opaca tiene otra clave. Los valores `Secret` no admiten formato de depuración ni serialización; los errores del SO se sustituyen por un error estable sin copiar sus datos.

Las pruebas normales usan un almacén en memoria. La prueba nativa escribe, lee, reemplaza y borra una entrada ficticia con un nombre único; no enumera ni toca otras credenciales. Ejecútala en la sesión nativa de cada SO, con el almacén disponible:

```sh
cargo test --workspace --locked native_store_round_trip_rotation_and_delete -- --ignored
```

En Windows, el aislamiento del ejecutor puede impedir el acceso al almacén; la prueba debe correr en la sesión del usuario. Esta prueba pasó en Windows el 2026-10-29. macOS y Linux todavía requieren evidencia propia; compilar o usar el fake no prueba esos almacenes.

Referencias: [URL de reqwest 0.12.28](https://docs.rs/reqwest/0.12.28/reqwest/struct.Url.html), [keyring 4.2.0 v1](https://docs.rs/keyring/4.2.0/keyring/v1/index.html), [Entry](https://docs.rs/keyring/4.2.0/keyring/v1/struct.Entry.html) y [errores del almacén](https://docs.rs/keyring/4.2.0/keyring/v1/enum.Error.html).

### Transporte y acceso OIDC (T05/T06)

`ApiTransport` usa reqwest 0.12.28 con un máximo de 10 s al conectar y 30 s por petición completa. No sigue redirecciones ni reintenta por sí solo. Limita peticiones a 1 MiB y respuestas a 8 MiB, incluso con transferencia por chunks. Los adaptadores de identidad y sincronización tienen rutas y DTO concretos; no reciben conexiones SQLite. Los errores conservan HTTP, código y correlación, sin copiar URL, mensajes remotos, detalles o errores crudos de red. Un HTTP 200 de lote conserva el resultado individual y `originalResult` de los duplicados.

`identity::oidc::OidcProvider` descubre el proveedor anunciado por API y usa openidconnect 4.0.1: Authorization Code, cliente público sin secreto, PKCE S256, state y nonce nuevos. La recepción escucha solo `127.0.0.1` en un puerto libre, acepta el camino `/oidc/callback` y termina por cancelación o a los cinco minutos. El proveedor debe aceptar esa redirección con puerto efímero para el cliente de desarrollo. El transporte comprueba HTTPS o loopback de desarrollo también en discovery, JWKS y token, con los mismos límites de cuerpo.

En Keycloak 26.8.1, añade `http://127.0.0.1/oidc/callback` a **Valid redirect URIs** del cliente público de desarrollo; conserva cualquier retorno fijo usado por sus otras pruebas. Keycloak comprueba el mismo retorno sin el puerto efímero para interfaces loopback ([implementación oficial](https://github.com/keycloak/keycloak/blob/26.8.1/services/src/main/java/org/keycloak/protocol/oidc/utils/RedirectUtils.java#L121)). Un retorno fijo como `http://127.0.0.1:18765/callback` no coincide con el camino del escritorio. Este ajuste pertenece al entorno OIDC; no requiere fijar el puerto del escritorio ni cambiar el contrato API. El usuario debe resolver los avisos de certificado del navegador externo.

Antes de devolver una sesión, el adaptador valida firma, emisor, audiencia del ID token, vencimiento, nonce y `at_hash` cuando existe. Usa el access token para API; nunca lo sustituye por el ID token. La API valida la audiencia de ese access token durante la emisión de la credencial de dispositivo. La renovación necesita que el proveedor haya emitido refresh token; si no lo emite, hace falta otro acceso. El nonce de un ID token renovado puede faltar; si aparece, debe coincidir con el original. La identidad no puede cambiar durante la renovación.

`OidcSession::persist` guarda las credenciales mediante el puerto seguro; el refresh token y su nonce original forman un bloque secreto. Si falla una escritura, intenta restaurar las entradas previas y devuelve `StorageUnavailable`; el llamante no debe aceptar el nuevo estado como guardado. No existe una transacción común entre entradas del almacén del SO. Cancelar antes de guardar deja intacta la sesión previa. `logout` solo borra claves locales explícitas; no llama a endpoints de revocación. T07 une esta sesión con acceso de cuenta y registro del dispositivo, según el apartado siguiente.

Las pruebas automáticas usan un navegador simulado y un proveedor loopback con firma RSA de prueba. La siguiente prueba abre el navegador externo y requiere API/OIDC y una cuenta de desarrollo. No guarda ni imprime sus tokens; pasó en Windows el 2026-11-02:

```sh
cargo test --workspace --locked manual_oidc_login_in_external_browser -- --ignored
```

En el ejecutor aislado de Windows, las pruebas HTTP/OIDC pueden fallar al conectar con loopback. Se deben ejecutar fuera del aislamiento; no se desactiva TLS ni se omiten pruebas para resolver ese fallo.

Referencias: [reqwest ClientBuilder 0.12.28](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html), [lectura por chunks](https://docs.rs/reqwest/0.12.28/reqwest/struct.Response.html#method.chunk), [openidconnect 4.0.1](https://docs.rs/openidconnect/4.0.1/openidconnect/#getting-started-authorization-code-grant-w-pkce), [validación de ID token](https://docs.rs/openidconnect/4.0.1/openidconnect/struct.IdToken.html#method.claims), [cliente HTTP async](https://docs.rs/openidconnect/4.0.1/openidconnect/trait.AsyncHttpClient.html), [renovación OIDC](https://openid.net/specs/openid-connect-core-1_0.html#RefreshTokenResponse) y [apertura con Opener 2.7.2](https://docs.rs/tauri-plugin-opener/2.7.2/tauri_plugin_opener/fn.open_url.html). Las firmas de API se contrastaron también con las fuentes instaladas de esas versiones.

### Activación de cuenta y revisión de C02 (T07)

`identity::application::AccountActivation::activate` recibe una sesión OIDC, el ID de la cuenta elegida y los puertos de reloj/IDs. Reutiliza la credencial de API de ese perfil, o la pide una sola vez si no existe. La credencial global no se duplica por cuenta: su clave incluye entorno, perfil, identidad y dispositivo de API. Otro registro del almacén seguro localiza el dispositivo actual y conserva el estado de emisión, sin contener el token. Los cambios de dispositivo no cambian los IDs ni el alcance de operaciones anteriores.

Antes del POST de emisión se guarda un marcador. Si se pierde la respuesta o falla el guardado del secreto, un reinicio no emite otra credencial de forma automática: devuelve `RecoveryRequired`. Cerrar sesión de forma explícita puede retirar ese estado local; el dispositivo remoto sigue bajo control de API y requiere revisión si quedó sin credencial. Durante un alta, otra alta o salida del mismo perfil devuelve `Busy`. Esa protección funciona en este proceso; la coordinación completa de sincronización pertenece a T13.

El caso consulta `/me`, verifica su usuario y pareja OIDC, recorre las páginas de `/accounts` hasta encontrar la cuenta y consulta `/access`. Solo `registration_required` permite el registro propio, con el `deviceId` autenticado, versión esperada cero, payload vacío y sin dependencias. El Outbox conserva el registro antes de enviarlo. Cada intento guarda `unknown` antes de HTTP; una respuesta perdida mantiene ID, contenido y huella. Un duplicado conserva su resultado original, incluso si fue rechazo. Tras aceptación, se vuelve a pedir `/access`; el resultado del POST no sustituye los permisos actuales.

Rust conserva rol, estado, `permissionEpoch` y `allowedActions` por cuenta/usuario/dispositivo. `cached_access` no hace HTTP. `allows_offline` exige acceso activo, acción permitida y una acción del catálogo que no figure como online-only en la copia de `capabilities.json`. No se duplican listas de acciones por rol. `revoked` o `inactive` impiden trabajo con esos permisos. Una denegación HTTP 401/403 deshabilita los permisos locales afectados hasta otra consulta válida; no borra Outbox. Ninguna petición mantiene el mutex o una transacción SQLite. El cierre local retira credenciales y selección de identidad; no revoca en servidor ni borra intenciones.

La implementación y las pruebas locales de T06/T07 están listas; el usuario dejó los recorridos reales para su revisión de C02. Para probar alta y cierre local, usa una cuenta y API/OIDC de desarrollo y ejecuta desde una terminal nativa:

```powershell
$env:NIGHTINGALE_API_BASE_URL = 'http://127.0.0.1:8000/'
$env:NIGHTINGALE_TEST_ACCOUNT_ID = '<UUIDv7 de la cuenta de desarrollo>'
cargo test --workspace --locked manual_c02_activation_and_local_logout -- --ignored --nocapture
```

El ID de cuenta es una entrada exclusiva de esta prueba, no configuración general del escritorio. La prueba abre el navegador, usa un perfil único y SQLite temporal, guarda el token de API en el almacén del SO y lo borra al cerrar sesión. Solo imprime el ID del dispositivo creado. Pasó en Windows el 2026-11-02. API conserva ese dispositivo para revisión; esta prueba no lo revoca. La matriz C06 verificó por separado la revocación del segundo dispositivo y su denegación posterior. Las pruebas reales de macOS Keychain y Linux Secret Service siguen pendientes por la excepción aprobada. El recorrido equivalente mediante IPC/UI se describe en T14/T15.

Fuentes: [transacciones de rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior), [keyring 4.2.0 v1](https://docs.rs/keyring/4.2.0/keyring/v1/index.html), [enum local con etiqueta en Serde](https://serde.rs/enum-representations.html#internally-tagged) y el [contrato API importado](../contracts/api-v1/nightingale-api.md#6-acceso-y-permisos).

### Envío del Outbox (T08)

`sync::dispatch::OutboxDispatcher::dispatch` ejecuta un intento sin temporizador ni bucle de fondo. Recibe una credencial de dispositivo y el contexto esperado. Refresca `/access` antes de cada lote; guarda la respuesta y detiene el envío ante revocación, inactividad, registro pendiente o cambio de época. HTTP 401/403 deshabilita los permisos locales afectados sin borrar intenciones. No mantiene SQLite durante HTTP.

`SqliteOperationStore::lease_batch` usa una transacción `IMMEDIATE` para seleccionar y reservar el lote. Solo considera la cuenta, usuario, dispositivo y época indicados, acciones permitidas e intenciones con huella válida. No asigna contexto a filas heredadas. `RegisterAccountDevice` conserva el recorrido propio de T07. Las dependencias deben estar aceptadas para el mismo actor/cuenta o aparecer antes en el lote; ciclos, referencias ausentes y dependencias rechazadas quedan pendientes sin bloquear operaciones independientes. El lote contiene hasta 100 operaciones, 1 MiB de cuerpo y 64 KiB por operación en UTF-8.

La reserva dura 60 segundos, por encima del timeout HTTP de 30 segundos. Guarda `in_flight`, incrementa `attempts` y fija el próximo intento antes de enviar. Impide otro lote para la misma instalación/cuenta mientras exista una reserva vigente. Un error de transporte deja `unknown` y espera entre la mitad y el total de un intervalo exponencial: empieza con 0,5–1 segundos y alcanza un máximo de 150–300 segundos. El generador de UUID existente aporta variación; no cambia IDs de operaciones. La fecha de reintento se conserva en SQLite. Al vencer una reserva o reabrir tras un cierre, se recupera `unknown` y se respeta esa fecha. Una respuesta tardía solo puede liberar su propio número de intento y reserva.

Desde T09, `dispatch` guarda los recibos antes de devolver el lote y la respuesta tipada para inspección. Si falla ese guardado, conserva la reserva para recuperar el resultado con el mismo ID, contenido y huella. No actualiza la proyección. El coordinador de T13 integra los casos sin abrir otra conexión de arranque mientras haya un envío activo; `sqlite::open` recupera reservas del proceso anterior. El ciclo IPC/UI se describe en T14/T15.

La selección carga los candidatos de ese ámbito y repite la búsqueda causal hasta 100 selecciones. Si el tamaño de las colas lo exige, se sustituirá por un recorrido paginado del grafo. Las pruebas usan reloj fijo y archivos SQLite temporales; el servidor HTTP de pruebas se comparte con las pruebas de activación.

Fuentes: [transacción de rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior), [UUID v7 de uuid 1.28.0](https://docs.rs/uuid/1.28.0/uuid/struct.Uuid.html#method.new_v7), serialización de bytes verificada en la fuente instalada `serde_json 1.0.154/src/ser.rs::to_vec` y el [contrato de operaciones y reintentos](../contracts/api-v1/nightingale-api.md#resultados-y-reintentos).

### Recibos, recuperación y revisión de C03 (T09)

`OperationResult::original` convierte cada respuesta en su recibo original. `duplicate` exige un `originalResult` con el mismo ID y puede resolver aceptación, conflicto, rechazo o fallo temporal. `entityVersion`, `state` y `error` deben existir en el JSON, aunque sean `null`; una respuesta incompleta no confirma la operación.

`SqliteOperationStore::reconcile` guarda el lote de recibos en una sola transacción. Comprueba alcance, ID, envoltorio, huella, número de intento y reserva. Un recibo repetido del mismo intento no escribe otra vez; otro intento o resultado terminal impide que una respuesta tardía cambie su estado. Los estados finales son `accepted`, `conflict` y `rejected`. `retryable_failure` queda `retryable` con otra fecha de intento. Los recibos conservan versión, estado remoto y error tipado, junto con la intención original; no guardan `message` ni `details` del error remoto. Un recibo no sustituye la proyección ni repite una escritura de negocio.

`OutboxDispatcher::recover` refresca acceso y consulta un solo `GET /sync/operations/{id}` para una operación `unknown` o `retryable` cuyo plazo se cumplió. Reserva esa consulta con las mismas reglas de concurrencia y aumenta el contador de intentos; ese contador incluye envíos y consultas de recuperación. No envía un POST ni cambia el ID. Un cierre durante la consulta permite recuperar el mismo resultado al reabrir. HTTP 401/403 deshabilita el acceso afectado; un fallo temporal conserva `unknown` con espera.

Un 404 de consulta o `RESULT_EXPIRED` conserva `unknown` y guarda `review_code` en la lectura local `SqliteOperationStore::operation_result`. La marca sobrevive a reapertura y excluye la operación de consultas y reenvíos automáticos. No crea otra intención. Una persona debe revisar visibilidad y estado actual antes de decidir el siguiente paso; IPC expone solo el resumen de revisión, sin payload ni estado remoto completo. Las operaciones independientes siguen disponibles.

Para revisar la demostración local de C03:

```powershell
cargo test --workspace --locked components::sync::dispatch::tests::reconciliation -- --nocapture
```

Las diez pruebas de esa matriz cubren todos los resultados originales y duplicados, recibos parciales, rollback, respuestas tardías, reintentos, reinicio tras confirmación remota con respuesta perdida, recuperación por GET y repetición del POST idéntico tras un timeout HTTP real. Ese timeout usa el mismo constructor de transporte con un plazo breve para la prueba. El servidor ficticio cuenta un solo efecto en ambas rutas de recuperación. También verifican 404/recibo vencido, conservación de intención/estado remoto y campos obligatorios ausentes. Esta evidencia usa datos sintéticos y HTTP loopback; no acredita API real ni revisión del usuario. El 2026-10-30 el usuario autorizó continuar con C02 y C03 pendientes hasta disponer de entorno manual. Esa excepción no aprueba esos checkpoints. La UI de diagnóstico y las revisiones se describen en T14/T15.

Fuentes: [transacciones y rollback de rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html#method.commit), [lectura opcional](https://docs.rs/rusqlite/0.40.2/rusqlite/trait.OptionalExtension.html), [campos de Serde 1.0.229](https://serde.rs/field-attrs.html#deserialize_with), fuente oficial instalada de `serde_json 1.0.154/src/value/mod.rs::to_value` y [contrato de resultados](../contracts/api-v1/nightingale-api.md#resultados-y-reintentos).

### Entrada, cursor y revisión de C04 (T10–T12)

`sync::remote::apply_groups` guarda proyección e Inbox en una transacción `IMMEDIATE`. Un ID de grupo u operación repetido no vuelve a llamar al proyector. El tipo contractual `entityType` elige uno de los 16 modelos de estado; no se deduce el modelo por sus campos. Se conservan claves compuestas, estado completo y ambas versiones de oferta. Una oferta con igual versión puede traer cambios de catálogo. Un cambio con versión menor no vuelve a proyectarse. `remove` retira la vista aceptada; las intenciones permanecen en Outbox.

Cada componente que implemente `RemoteProjector` recibe el cambio tipado y la misma transacción. `project` actualiza solo su vista aceptada; `reset` retira solo esa vista del ámbito cuenta/usuario/dispositivo al instalar una captura. Ninguno escribe intenciones, hace HTTP o produce efectos externos. Un error debe propagarse para que SQLite revierta también el proyector. El proyector vacío `()` permite verificar el motor antes de implementar componentes de negocio.

`ChangeDownloader::download` refresca acceso por página y consulta con el cursor confirmado. No mantiene SQLite durante HTTP. Confirma la página, Inbox, proyectores y `nextCursor` juntos, tras comprobar de nuevo acceso y cursor previo. Continúa con `hasMore`, incluso con grupos vacíos. Si no hay cursor, vence o cambia la época, devuelve `RecoveryRequired` sin borrar trabajo. Una respuesta incompleta o sin avance con `hasMore` falla sin avanzar. Los parámetros se codifican con el transporte, incluidos caracteres reservados del cursor.

`SnapshotRecovery::rebuild` crea una captura con ventana por defecto (`{}`), o reanuda su descarga persistida en páginas de 100. Cada página guarda elementos y número de página juntos; comprueba total, identidad, época y posición. Conserva el orden de inserción que entrega API. Antes de instalar, vuelve a leer los metadatos de API para comprobar vigencia y generación: no decide la vigencia a partir del reloj local. API sigue siendo la autoridad sobre vencimiento y alcance.

La instalación retira la vista remota e Inbox anteriores de esa instalación, ejecuta `reset`/`project` y guarda `baseCursor` con su ventana en una sola transacción. No modifica Outbox, resultados ni conflictos. Un fallo conserva la vista/cursor previos y permite repetir la instalación. El valor local `i64::MAX` de `next_page` marca instalación confirmada; nunca se envía como número de página. Si el proceso termina después del commit o falla DELETE, la siguiente llamada reintenta solo la liberación. No reinstala la captura ni retrocede un cursor incremental posterior. `cleanup_pending` informa de ese pendiente.

404 de captura o `SNAPSHOT_EXPIRED`/`SNAPSHOT_SCOPE_CHANGED` retiran solo su preparación y devuelven `RecoveryRequired`. Otra llamada explícita puede crear otra captura. `SnapshotMode::Restart` permite descartar preparación de una época anterior sin tocar la vista actual. Una respuesta de metadatos incompatible se conserva como fallo, sin instalarla. Una captura creada cuya respuesta se pierde, o descartada por reinicio explícito, puede quedar en API hasta que venza; no tiene efectos de negocio.

Para revisar la matriz local con datos sintéticos:

```powershell
cargo test --workspace --locked components::sync::remote::tests
cargo test --workspace --locked components::sync::download::tests
cargo test --workspace --locked components::sync::snapshot::tests
pnpm quality
git diff --check
```

Las pruebas cubren repetición, fallo de proyector/cursor, permisos cambiados durante HTTP, captura de 101 elementos, reinicio antes y después de instalar, liberación fallida, vencimiento antes de instalar y reconstrucción de perfil vacío seguida de cambios incrementales. La captura de prueba usa datos ficticios; no acredita API/OIDC de desarrollo ni revisión del usuario. La instalación carga los elementos preparados en memoria; si el tamaño real exige otro límite, deberá aplicar por lectura acotada dentro de la misma transacción. T13 aporta el coordinador y T14/T15 describen el ciclo IPC/UI. El usuario autorizó continuar el 2026-10-30 dejando C04 pendiente de revisión manual.

Fuentes: [rusqlite 0.40.2, transacciones](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html), [OptionalExtension](https://docs.rs/rusqlite/0.40.2/rusqlite/trait.OptionalExtension.html), [Serde 1.0.229, etiqueta y contenido](https://serde.rs/enum-representations.html#adjacently-tagged), [campos nullable obligatorios](https://serde.rs/field-attrs.html#deserialize_with), [reqwest 0.12.28, consulta codificada](https://docs.rs/reqwest/0.12.28/reqwest/struct.RequestBuilder.html#method.query), fuente oficial instalada de `serde_json 1.0.154` y [contrato fijo de entrada/capturas](../contracts/api-v1/nightingale-api.md#9-captura-inicial-descarga-y-avisos).

### Coordinador nativo (T13)

La app conserva un `SyncCoordinator` por perfil. `SyncEngine::run` es la entrada Rust para `Startup`, `Reconnect` y `Manual`. `identity::desktop` aporta credencial y contexto; el coordinador no emite tokens ni decide un perfil por su cuenta. El proyector compuesto actual es `()` hasta que los componentes de negocio registren los suyos. El arranque local no necesita configurar API ni ejecutar HTTP; el supervisor remoto solo se inicia si existe identidad guardada y configuración válida.

Por cuenta/usuario/dispositivo, un mutex asíncrono permite un solo ciclo. Las llamadas concurrentes reciben el mismo informe al terminar. La época no cambia esa clave: un permiso nuevo no permite iniciar otro envío junto al anterior ni reasignar intenciones antiguas. Una petición manual que llega durante HTTP pide otra pasada dentro del mismo ciclo; espera a la activa y no pierde su reintento. Cancelar una llamada libera el mutex y retira la fase en memoria; las reservas e intenciones permanecen en SQLite para recuperación.

El ciclo refresca acceso, consulta por GET los resultados desconocidos cuyo plazo venció, envía lotes causales y descarga cambios. Los resultados temporales ya conocidos vuelven por POST cuando corresponde; consultar siempre su recibo no volvería a ejecutar la intención. Un reintento manual elimina solo la espera de operaciones `unknown`/`retryable` de la época actual: no altera contenido, IDs, contador, reservas vigentes o marcas `NOT_FOUND`/`RESULT_EXPIRED`. La selección de desconocidos carga sus IDs una vez; si la cola lo exige, se sustituirá por páginas.

Ante una época nueva se usa el acceso recién confirmado, sin cambiar el contexto guardado de operaciones anteriores. La ausencia o invalidez de cursor lleva a recuperación por captura con reintentos acotados. Una captura en preparación se reanuda; una instalada con liberación pendiente solo reintenta DELETE. Si ese cursor venció, el ciclo pide otra captura y continúa con cambios. Cada petición usa los casos de T08–T12; ningún mutex de SQLite cubre HTTP.

`SyncCoordinator::status` no hace HTTP ni carga payloads: lee cantidades de Outbox, conflictos/rechazos, revisiones, operaciones de otra época, próxima fecha de intento, presencia de cursor y captura pendiente. Ese trabajo durable sobrevive a reapertura y permite reconstruir el estado con un coordinador nuevo. Las fases en curso y la conectividad son transitorias; no se presentan como evidencia persistida. `Complete` indica que terminó el intercambio; los contadores aún pueden señalar conflictos, revisiones o intenciones bloqueadas. Los fallos publican códigos estables, sin mensajes remotos, secretos o datos de pacientes.

Comprobación local:

```powershell
cargo test --workspace --locked components::sync::coordinator::tests
pnpm quality
git diff --check
```

La prueba real `manual_t13_cycle_with_development_api` permanece ignorada por defecto y pasó en Windows el 2026-11-02. Requiere una URL de API/OIDC de desarrollo confirmada y una cuenta con datos sintéticos. Con esas entradas no secretas disponibles, se ejecuta desde terminal nativa:

```powershell
cargo test --workspace --locked manual_t13_cycle_with_development_api -- --ignored --nocapture
```

La prueba abre el navegador, activa una instalación con perfil único, ejecuta un ciclo completo sobre SQLite temporal y cierra sesión para retirar sus credenciales locales. El servidor conserva el dispositivo creado para revisión/revocación posterior. No imprime tokens. La verificación externa tiene evidencia propia, distinta de las pruebas loopback.

Fuentes: [runtime de Tauri 2.12.4](https://docs.rs/tauri/2.12.4/tauri/async_runtime/index.html), [mutex de Tokio 1.54.0](https://docs.rs/tokio/1.54.0/tokio/sync/struct.Mutex.html#method.lock) y [transacciones de rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior). Las páginas versionadas de Tauri/Tokio no respondieron al navegador: se verificaron la reexportación y `Mutex::lock` en sus fuentes oficiales instaladas (`tauri-2.12.4/src/async_runtime.rs`, `tokio-1.54.0/src/sync/mutex.rs`), sin cambiar versiones ni añadir Tokio como dependencia directa.

### Avisos SSE, IPC y ventana (T14/T15)

`SyncEngine::listen_once` abre `/sync/events` con el último cursor aplicado. El parser admite chunks, UTF-8, CRLF y datos multilínea hasta 16 KiB. `changes_available` solo despierta un ciclo: ni su cursor anunciado ni el ID SSE se guardan como cursor. `heartbeat` mantiene la conexión; `access_revoked` deshabilita el acceso y detiene el supervisor. La lectura inactiva vence a los 45 segundos, sin timeout total del stream. `identity::desktop` evita supervisores duplicados y reconecta con espera de 3 a 300 segundos; logout cancela y espera la tarea antes de borrar credenciales.

Los comandos IPC específicos son `identity_get_status`, `identity_begin_login`, `identity_complete_login`, `identity_cancel_login`, `identity_list_accounts`, `identity_activate_account`, `identity_logout` y `sync_now`. Todos tienen permisos explícitos en `src-tauri/capabilities/default.json`; `build.rs` declara sus comandos. `src/shared/api/identity.ts` y `sync.ts` son la única frontera de React hacia esos casos. `CorePanel` presenta acceso, actividad, espera, resultado y revisiones con DTO camelCase, sin tokens, payloads, estado remoto completo ni SQL. La hora de última sincronización es de esta ejecución; el trabajo durable se lee de SQLite.

El recorrido manual de C05 pasó en Windows el 2026-11-02 y el usuario aprobó el checkpoint ese mismo día. Con API/OIDC de desarrollo y `NIGHTINGALE_API_BASE_URL` configurados, se ejecuta `pnpm tauri dev` o el build de escritorio, se inicia acceso, se elige una cuenta sintética y se comprueban sincronización manual, corte y reconexión. La interfaz no debe afirmar aceptación remota mientras una intención esté pendiente o su resultado sea desconocido. La evidencia actual incluye una sonda Rust guardada offline, reapertura, confirmación del mismo ID y logout desde Tauri. Un proxy loopback temporal controló solo la conexión del escritorio; la API original siguió activa.

Comprobación local del 2026-10-30: el ejecutable compilado abrió en Windows sin configuración API, mostró diagnóstico IPC con esquema 4 y acceso deshabilitado, y volvió a responder tras cerrar y reabrir. La base real conservó `user_version=4`, integridad `ok`, Outbox vacío y sin identidad; no se modificó. Esta evidencia no cubre persistencia de una operación pendiente ni el recorrido remoto. Las pruebas automatizadas de reapertura y Outbox siguen siendo la evidencia local para esos casos.

Fuentes: [reqwest 0.12.28, `read_timeout`](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html#method.read_timeout), [lectura por chunks](https://docs.rs/reqwest/0.12.28/reqwest/struct.Response.html#method.chunk), [eventos SSE de WHATWG](https://html.spec.whatwg.org/multipage/server-sent-events.html), [comandos Tauri 2](https://v2.tauri.app/develop/calling-rust/) y [permisos/capacidades](https://v2.tauri.app/security/capabilities/).

### Contratos y matriz de dos perfiles (T16/T17)

Las pruebas de consumidor leen la copia fija `api-v1.1`, comprueban los hashes del manifiesto, referencias, rutas consumidas, envoltorio, estados remotos y SSE. Sus fixtures estructurales detectan deriva sin red externa ni credenciales; no reemplazan la validación completa de negocio, fechas o zonas de API. Nunca se editan los artefactos importados para hacer pasar una prueba.

La matriz local usa dos archivos SQLite, perfiles y almacenes secretos falsos aislados. Activa dos dispositivos y ejerce una operación `RegisterPatient` sintética como sonda, sin implementar el componente Pacientes. Cubre indisponibilidad, commit remoto con respuesta perdida, GET tras reapertura, duplicado, descarga en ambos perfiles, conflicto, revocación, cursor/captura vencidos y SSE caído. Comprueba un solo efecto remoto y conservación de intenciones. La matriz externa descrita abajo pasó el 2026-11-02; sus fallos controlados y límites se registran por separado.

La prueba optativa siguiente usa los adaptadores Rust reales contra una API local. Requiere el retorno OIDC y, si corresponde, el PEM indicados arriba:

```powershell
cargo test --workspace --locked manual_c06_two_profiles_with_development_api -- --ignored --nocapture
```

Abre el navegador una vez, crea dos dispositivos y un paciente ficticio, actualiza ese paciente y revoca solo el segundo dispositivo creado. Usa dos SQLite temporales y almacenes secretos en memoria; no sustituye la comprobación del almacén nativo ni el recorrido Tauri. Si el usuario tiene una sola cuenta propietaria la selecciona; si tiene varias, exige `NIGHTINGALE_TEST_ACCOUNT_ID`. Conserva el paciente y los dispositivos en API para revisión. No borra datos de negocio.

Para el recorrido Tauri con trabajo pendiente, `manual_c05_save_offline_probe` exige `NIGHTINGALE_TEST_DESKTOP_DB` con la ruta absoluta a la base de prueba ya abierta por el escritorio y `NIGHTINGALE_TEST_ACCOUNT_ID`. Comprueba cuenta y permiso offline, guarda una sonda sintética en el Outbox mediante Rust y termina sin HTTP. Crea datos de prueba en esa base: úsala solo en el recorrido autorizado. Corta la conexión antes de ejecutarla, comprueba el contador pendiente, cierra/reabre y reconecta. Después de ejecutar `pnpm quality`, genera `pnpm build:desktop` antes de abrir el ejecutable: Cargo puede construir un binario sin `custom-protocol` para las pruebas de integración. Durante la revisión del build ya abierto, se puede ejecutar la prueba optativa directamente desde el ejecutable de pruebas generado en `target/debug/deps`, sin reconstruir el ejecutable principal.

Verifica guardado/reapertura, conexión rechazada sin detener la API del usuario, respuesta confirmada descartada antes de reconciliar, recuperación por GET tras reapertura, duplicado desde el otro dispositivo, versión en conflicto, reconstrucción por cursor de firma inválida, aviso SSE de revocación y conservación del conflicto. La respuesta descartada modela la pérdida entre HTTP y el commit local; no corta la conexión del servidor tras su commit. El cursor inválido ejerce `CURSOR_EXPIRED`, no espera 90 días. El vencimiento temporal de capturas y la caída del servicio SSE siguen en las matrices automáticas locales, salvo evidencia externa adicional. Al terminar, el logout local elimina los secretos en memoria y se revisan los archivos SQLite en busca de los tokens de esa ejecución; el arnés no escribe tokens en sus mensajes.

```powershell
cargo test --workspace --locked components::sync::contract_tests
cargo test --workspace --locked components::sync::matrix_tests
```

### Observabilidad y colaboración (T18/T19)

`platform::observability::Event` escribe JSON en stderr con tipo, resultado, instante, duración, IDs de operación/causación/correlación, código y contadores. La interfaz cerrada no admite mensaje remoto, URL, cursor, SQL, token, payload, clave de entidad ni estado remoto. HTTP, recibos y ciclos emiten esos metadatos; un fallo del escritor no cambia el resultado del caso de uso. El 2026-11-02 se inspeccionaron también errores de API real y desconexión, recibos y ciclos del recorrido Tauri: catorce registros de prueba sin patrones de credenciales. Las pruebas de whitelist, redacción y destino fallido siguen siendo locales.

Las guías de [sync](../technical/components/sync.md), [identity](../technical/components/identity.md), [devices](../technical/components/devices.md), [platform](../technical/components/platform.md) y [shared kernel](../technical/components/shared-kernel.md) resumen propiedad, API, puertos, invariantes y pruebas. Los [contratos de los siguientes tracks](core-next-tracks.md) muestran cómo añadir una operación, proyector, comando y migración desde `0005`, con un ejemplo sintético completo. Esos tracks requieren nuevos planes y no forman parte de esta entrega.

```powershell
cargo test --workspace --locked platform::observability
pnpm quality
pnpm build:desktop
git diff --check
```

C06 quedó aprobado por el usuario el 2026-11-02 tras la verificación local, el build, la matriz real de dos perfiles y el recorrido Tauri. Se conservan los límites documentados de macOS/Linux y pruebas temporales; el cierre no atribuye evidencia a entornos o esperas no ejecutados.

## Requisitos

| Sistema de referencia | Arquitectura | Requisitos nativos |
|---|---|---|
| Windows 11 | x64 | Microsoft C++ Build Tools con **Desktop development with C++**, Windows SDK y WebView2 Runtime. Usa Rust MSVC, no GNU. |
| macOS 15 | ARM64 o Intel | Xcode Command Line Tools; terminal y herramientas de la misma arquitectura. |
| Fedora 44 | x64 | Compilador C/C++, pkg-config y bibliotecas Tauri indicadas abajo. |

La matriz de referencia no incluye todas las versiones o distribuciones. Cada equipo ejecuta los controles locales. Cada Mac compila para su arquitectura. Esta base no incluye jobs de CI.

En macOS, instala las herramientas si aún no están presentes:

```sh
xcode-select --install
```

En Fedora 44:

```sh
sudo dnf install gcc gcc-c++ make pkgconf-pkg-config webkit2gtk4.1-devel openssl-devel curl wget file libappindicator-gtk3-devel librsvg2-devel libxdo-devel
```

Consulta la [guía oficial de requisitos Tauri 2](https://v2.tauri.app/start/prerequisites/) para instalar C++/WebView2 en Windows o revisar herramientas de otro sistema. No hacen falta herramientas de firma ni de instaladores para esta base.

## Herramientas fijadas

| Herramienta | Versión | Fuente en el repositorio |
|---|---|---|
| Node.js | 24.18.0 | `.node-version` y `package.json` |
| pnpm | 11.17.0 | `packageManager` y `engines` en `package.json` |
| Rust/Cargo | 1.99.1 | `rust-toolchain.toml`, con rustfmt y Clippy |

Instala [Node.js 24.18.0](https://nodejs.org/en/blog/release/v24.18.0) para tu sistema y arquitectura. `.node-version` declara la versión; por sí solo no la instala ni cambia el Node de una terminal abierta. Después instala pnpm:

```sh
npm install --global pnpm@11.17.0
```

Instala [rustup](https://www.rust-lang.org/tools/install). En Windows elige `x86_64-pc-windows-msvc`. Reinicia la terminal tras instalar herramientas y, desde la raíz del proyecto, comprueba:

```sh
node --version
pnpm --version
rustup show
rustc -vV
cargo --version
```

`rustup show` carga el toolchain del archivo del proyecto; puede descargarlo en la primera ejecución. Comprueba también el host nativo en `rustc -vV`. No sustituyas la versión fijada por `stable` para resolver un fallo local.

## Primera ejecución

Obtén una copia de la rama compartida del equipo y entra en su raíz, donde están `package.json` y `Cargo.toml`. El remoto del equipo es [odiseo0/nightingale](https://github.com/odiseo0/nightingale).

```sh
pnpm install --frozen-lockfile
pnpm quality
pnpm tauri dev
```

El primer comando instala desde `pnpm-lock.yaml`. Cargo usa el lockfile de raíz; los controles Rust pasan `--locked`. No borres lockfiles para resolver errores.

En la ventana:

1. Pulsa Tab hasta **Consultar diagnóstico** y comprueba el contorno de foco.
2. Pulsa Enter. Debe mostrar versión `0.1.0`, tu sistema y tu arquitectura desde Rust.
3. Cierra la ventana y espera a que termine el proceso de desarrollo.
4. Abre otra vez con `pnpm tauri dev`. Debe aceptar la misma base y su esquema.

La vista web de `pnpm dev` permite trabajar en la interfaz, pero no tiene el proceso Rust. Allí el diagnóstico muestra un error previsto; no indica que el escritorio esté roto.

## Calidad y compilación

`pnpm quality` ejecuta en orden:

```sh
pnpm build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

La cadena se detiene al primer fallo. `pnpm build` comprueba TypeScript y compila Vite. El control no corrige fuentes ni actualiza lockfiles. Las pruebas SQLite crean y retiran sus propias carpetas temporales; no usan la base de la app. `cargo fmt --all` sí corrige el formato y se ejecuta aparte cuando corresponde.

```sh
pnpm build:desktop
```

Este comando ejecuta `tauri build --debug --no-bundle`: compila los recursos locales y el escritorio, sin instaladores ni publicación. El ejecutable queda en `target/debug/nightingale.exe` en Windows o `target/debug/nightingale` en macOS/Linux. Ciérralo antes de volver a compilar en Windows.

## Datos locales

### Convención de migraciones

Cada migración vive en `src-tauri/migrations/` como `NNNN_<componente>_<slug>.sql`, con `NNNN` secuencial y sin huecos. El runner exige que cada archivo fije `PRAGMA user_version` con su propio número; un historial inválido o una versión futura se rechazan sin modificar el archivo. Antes de ejecutar un plan que añade una migración, reserva su número en el `todo.md` del plan para evitar colisiones entre trabajos paralelos. Las migraciones se aplican en orden dentro de una sola transacción.

Rust obtiene la carpeta mediante `app.path().app_data_dir()` de Tauri y crea `nightingale.sqlite` dentro. Con los valores por defecto del sistema:

| Sistema | Carpeta de la base |
|---|---|
| Windows | `%APPDATA%/com.pedro.nightingale/` |
| macOS | `~/Library/Application Support/com.pedro.nightingale/` |
| Linux | `$XDG_DATA_HOME/com.pedro.nightingale/`, o `~/.local/share/com.pedro.nightingale/` si no se define XDG_DATA_HOME |

La app no guarda la base en el repositorio. No comparte archivos SQLite entre dispositivos. Desarrollo y el ejecutable compilado usan el mismo identificador y la misma base local: no son perfiles de datos aislados.

La migración `0001_schema` crea solo `schema_migrations`, marca `user_version=1` y el identificador de la aplicación. Reabrir no repite la migración. Rust rechaza versiones no compatibles, bases ajenas, historiales inválidos o archivos dañados; no los elimina ni los reemplaza. Si falla la apertura, el arranque termina y la terminal de desarrollo muestra un mensaje sin rutas personales ni datos SQL.

No se ha activado WAL ni cifrado. rusqlite 0.40.2 incluye SQLite al compilar, así que no debes instalar una biblioteca SQLite aparte. Su soporte SQLCipher deja abierta esa opción; aún hay que definir claves y conversión del archivo antes de admitir datos reales. No borres ni cambies una base para ocultar un fallo.

## Fallos frecuentes

| Síntoma | Qué revisar |
|---|---|
| Node o pnpm no coincide con `engines` | Comprueba versiones en la terminal actual y las rutas de las herramientas. Instala las versiones fijadas. |
| Cargo no encuentra linker o SDK | Revisa C++ Build Tools/MSVC en Windows, Xcode tools en macOS o los paquetes Fedora. |
| Falta WebKitGTK o pkg-config en Linux | Instala la lista de requisitos de Fedora 44. No uses WebKitGTK 4.0 en lugar de 4.1. |
| Fallo de red o permisos al instalar | Comprueba red y acceso al almacén npm/Cargo. Conserva el error; no cambies dependencias por una falla del entorno. |
| Puerto 1420 ocupado | Cierra la sesión de Nightingale anterior. Vite usa un puerto fijo para conectar con Tauri. |
| EBUSY o acceso denegado al compilar | Cierra el ejecutable anterior y evita compilar dos copias a la vez. `target/` ya queda fuera del watcher Vite. |
| Diagnóstico falla en el navegador | Abre `pnpm tauri dev`; `pnpm dev` no tiene IPC. |
| No se puede abrir la base local | Cierra otras instancias; revisa permisos y la versión de la app que creó el esquema. Conserva el archivo y pide revisión; no borres la base. |

## Verificar otra plataforma o un clon limpio

Usa un clon nuevo sin copiar `node_modules`, `dist`, `target`, archivos `.env` o bases. Sigue esta guía, ejecuta calidad y compilación, y prueba la ventana con datos sintéticos. Para una revisión independiente, otro colaborador puede repetir el recorrido. El usuario aceptó el cierre de esta base sin esa revisión el 2026-10-21.

Registra en [C05](../../tasks/P-20261019-01-desktop-foundation/todo.md#c05-después-de-t10t11) el sistema/versión, arquitectura, revisión Git, comandos y códigos de salida, resultado del diagnóstico, reapertura SQLite y responsable. No incluyas rutas personales, bases ni registros con datos privados.

El propietario pidió retirar los jobs de GitHub Actions el 2026-10-21. La verificación continúa con los comandos de esta guía en cada equipo; la retirada no demuestra que la app abra en plataformas aún no probadas. El usuario aceptó el cierre sin pruebas manuales macOS/Fedora ni revisión de otro colaborador el 2026-10-21. No consta que estas comprobaciones se realizaran.

## Fuentes técnicas

- [Tauri 2: requisitos](https://v2.tauri.app/start/prerequisites/).
- [Tauri 2.12.4: carpetas de datos](https://github.com/tauri-apps/tauri/blob/tauri-v2.12.4/crates/tauri/src/path/desktop.rs).
- [rusqlite 0.40.2: SQLite incluido y SQLCipher](https://github.com/rusqlite/rusqlite/blob/v0.40.2/README.md).
- [pnpm: scripts](https://pnpm.io/cli/run) e [instalación fija](https://pnpm.io/cli/install#--frozen-lockfile).

La selección de paquetes Fedora se cotejó con la guía Tauri 2 y el catálogo oficial de [WebKitGTK 4.1 para Fedora 44](https://packages.fedoraproject.org/pkgs/webkitgtk/webkit2gtk4.1-devel/fedora-44.html) y [libappindicator para Fedora 44](https://packages.fedoraproject.org/pkgs/libappindicator/libappindicator-gtk3-devel/fedora-44.html). La ejecución local de estos comandos en Fedora sigue pendiente.
