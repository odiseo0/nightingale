# Desarrollo del escritorio

Esta guía cubre la base local de Nightingale. No requiere servicios web, PostgreSQL, n8n, Docker ni configuración privada. La base SQLite contiene solo metadatos del esquema. Las funciones de negocio siguen pendientes.

**Revisión documental del 2026-10-23:** alineación de permisos, sin cambiar las rutas del escritorio ni repetir sus comprobaciones de ejecución.

## Límites de esta base y de las integraciones futuras

La [arquitectura vigente](../decisions/transactional-sync-three-projects.md) asigna la operación local a Rust/SQLite y la aceptación remota a Nightingale API por HTTPS. El escritorio no tiene acceso SQL o credenciales PostgreSQL. Nightingale Patient Hub tiene web y API propias y usa Nightingale API para consultar y modificar citas. Ambos tendrán repositorios separados del escritorio.

Esta base aún no implementa sincronización, cliente del Hub o identidad. Sus comandos actuales no requieren servicios cloud. Al planificar la integración, cada colaborador usará un entorno API aislado con su base remota de desarrollo; las credenciales PostgreSQL pertenecen al servidor API y a la administración autorizada, no al cliente de escritorio. No se exigirá Docker. Recordatorios, integración con n8n y analítica pertenecen a módulos/workers de Nightingale API. La instancia n8n de desarrollo seguirá siendo remota y su trabajo no bloqueará transacciones de sincronización.

No cambies permisos sin conexión ni aceptes una operación pendiente por su fecha local. D06, del repositorio de API, confirma OIDC externo y permisos vigentes al aceptar: si falta permiso, API rechaza sin efecto y el escritorio conserva la intención para revisión. Proveedor, sesiones, vínculo de dispositivos, orden concurrente de aceptación/revocación y recorrido de revisión siguen pendientes en el diseño de acceso de API.

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
