# Desarrollo del escritorio

Esta guía cubre la base local de Nightingale. No requiere servicios web, PostgreSQL, n8n, Docker ni configuración privada. La base SQLite contiene solo metadatos del esquema. Las funciones de negocio siguen pendientes.

**Revisión documental del 2026-10-23:** alineación de permisos, sin cambiar las rutas del escritorio ni repetir sus comprobaciones de ejecución.

## Límites de esta base y de las integraciones futuras

La [arquitectura vigente](../decisions/transactional-sync-three-projects.md) asigna la operación local a Rust/SQLite y la aceptación remota a Nightingale API por HTTPS. El escritorio no tiene acceso SQL o credenciales PostgreSQL. Nightingale Patient Hub tiene web y API propias y usa Nightingale API para consultar y modificar citas. Ambos tendrán repositorios separados del escritorio.

Esta base aún no implementa sincronización, cliente del Hub o identidad. Sus comandos actuales no requieren servicios cloud. Al planificar la integración, cada colaborador usará un entorno API aislado con su base remota de desarrollo; las credenciales PostgreSQL pertenecen al servidor API y a la administración autorizada, no al cliente de escritorio. No se exigirá Docker. Recordatorios, integración con n8n y analítica pertenecen a módulos/workers de Nightingale API. La instancia n8n de desarrollo seguirá siendo remota y su trabajo no bloqueará transacciones de sincronización.

No cambies permisos sin conexión ni aceptes una operación pendiente por su fecha local. D06, del repositorio de API, confirma OIDC externo y permisos vigentes al aceptar: si falta permiso, API rechaza sin efecto y el escritorio conserva la intención para revisión. Proveedor, sesiones, vínculo de dispositivos, orden concurrente de aceptación/revocación y recorrido de revisión siguen pendientes en el diseño de acceso de API.

## Motor transversal y colaboración

El plan `P-20261029-03-core-engine` y su [ADR](../decisions/desktop-core-engine.md) definen los componentes `identity`, `devices`, `platform` y `sync`, la propiedad de sus tablas y las migraciones reservadas `0003`/`0004`. Hasta que esas tareas se implementen, la aplicación sigue sin identidad ni sincronización remota; una decisión documental no acredita el recorrido.

Para el trabajo local del motor, `NIGHTINGALE_API_BASE_URL` selecciona explícitamente la API de desarrollo. No existe un destino remoto por defecto. El emisor, cliente público y audiencia OIDC se obtienen de `/v1/auth/config`; no se duplican en variables ni se guardan credenciales en `.env`. Release exige HTTPS y solo desarrollo/pruebas admiten HTTP loopback.

Cada componente modifica únicamente sus tablas y expone puertos para los demás. Antes de abrir tracks paralelos, fija en el plan DTO, acciones, proyectores, comandos, propietarios y migraciones. El integrador registra composición y permisos en los puntos calientes enumerados en `CONTRIBUTING.md`. Los artefactos de `docs/contracts/api-v1/` se sustituyen como una exportación completa del repositorio de API, nunca mediante ediciones locales.

### Configuración y credenciales del motor (T04)

`platform::config::ApiConfig::from_environment` lee únicamente `NIGHTINGALE_API_BASE_URL`, cuando un caso remoto necesita la configuración. No carga `.env`, no tiene URL por defecto y no exige esa variable para abrir SQLite o ejecutar los casos locales. La configuración rechaza usuario/contraseña, query, fragmento y protocolos distintos de HTTPS. Solo los builds de desarrollo admiten HTTP en `localhost` o una IP loopback. Versión del cliente y plataforma proceden del build y del SO; el descubrimiento OIDC corresponde al flujo de acceso posterior.

Ejemplo ficticio para una API local ya iniciada, en la misma terminal del escritorio:

```powershell
$env:NIGHTINGALE_API_BASE_URL = 'http://127.0.0.1:8000/'
```

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

Antes de devolver una sesión, el adaptador valida firma, emisor, audiencia del ID token, vencimiento, nonce y `at_hash` cuando existe. Usa el access token para API; nunca lo sustituye por el ID token. La API valida la audiencia de ese access token durante la emisión de la credencial de dispositivo. La renovación necesita que el proveedor haya emitido refresh token; si no lo emite, hace falta otro acceso. El nonce de un ID token renovado puede faltar; si aparece, debe coincidir con el original. La identidad no puede cambiar durante la renovación.

`OidcSession::persist` guarda las credenciales mediante el puerto seguro; el refresh token y su nonce original forman un bloque secreto. Si falla una escritura, intenta restaurar las entradas previas y devuelve `StorageUnavailable`; el llamante no debe aceptar el nuevo estado como guardado. No existe una transacción común entre entradas del almacén del SO. Cancelar antes de guardar deja intacta la sesión previa. `logout` solo borra claves locales explícitas; no llama a endpoints de revocación. T07 une esta sesión con acceso de cuenta y registro del dispositivo, según el apartado siguiente.

Las pruebas automáticas usan un navegador simulado y un proveedor loopback con firma RSA de prueba. La siguiente prueba abre el navegador externo y requiere API/OIDC y una cuenta de desarrollo. No guarda ni imprime sus tokens; todavía no se ha ejecutado:

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

El ID de cuenta es una entrada exclusiva de esta prueba, no configuración general del escritorio. La prueba abre el navegador, usa un perfil único y SQLite temporal, guarda el token de API en el almacén del SO y lo borra al cerrar sesión. Solo imprime el ID del dispositivo creado. API conserva ese dispositivo para la revisión de revocación; la prueba no lo revoca. C02 debe revisar también una denegación/revocación con API de desarrollo. Las pruebas reales de macOS Keychain y Linux Secret Service siguen pendientes por la excepción aprobada. No se ha añadido UI ni IPC de acceso: pertenecen a T15.

Fuentes: [transacciones de rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior), [keyring 4.2.0 v1](https://docs.rs/keyring/4.2.0/keyring/v1/index.html), [enum local con etiqueta en Serde](https://serde.rs/enum-representations.html#internally-tagged) y el [contrato API importado](../contracts/api-v1/nightingale-api.md#6-acceso-y-permisos).

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
