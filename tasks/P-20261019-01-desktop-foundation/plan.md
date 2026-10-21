# P-20261019-01-desktop-foundation: Base de desarrollo del escritorio

**Status:** Completed
**Created:** 2026-10-19
**Updated:** 2026-10-21
**Task list:** [todo.md](todo.md)

## Overview

Preparar el proyecto de escritorio para que un colaborador pueda instalar sus dependencias, abrir la aplicación, comprobar React → Tauri → Rust y ejecutar los mismos controles que el equipo. La base deberá cubrir Windows, macOS y Linux desde el inicio.

El usuario aclaró que los servicios web quedan fuera de este trabajo inicial. Elegir el lenguaje del servidor no bloquea este plan. La API, Patient Hub, recordatorios, n8n, PostgreSQL y analítica tendrán planes posteriores. Esto delimita la base actual; no elimina esas partes del producto descrito en `docs/`.

La entrega incluye una base SQLite vacía con migraciones comprobables, sin pacientes ni reglas de citas. No incluye pantallas de negocio, sincronización real, identidad cloud, diseño visual definitivo, instaladores firmados ni publicación.

El usuario aprobó comenzar este plan el 2026-10-19: «Vale, me gustaría que trabajásemos para lograr el objetivo de hacerlo trabajable para mi equipo. Puedes comenzar, por favor». Se ejecutará una tarea cada vez y se conservarán las revisiones de los checkpoints.

### Diagnóstico del 2026-10-19

**Resultado: todavía no está listo para que el equipo empiece sin bloqueos.** La documentación permite preparar la base, pero aún no existe una instalación reproducible ni una compilación verificada.

| Prioridad | Hallazgo y evidencia | Efecto | Tratamiento |
|---|---|---|---|
| Bloqueo actual | `Cargo.toml` raíz declara un paquete sin target; `crates/main.rs` no ocupa una ruta descubierta por Cargo ni figura como target explícito. Fallan los metadatos tanto desde raíz como desde `src-tauri/`. | Cargo no puede cargar el proyecto Tauri. | T03 |
| Bloqueo de coordinación | `AGENTS.md` conserva el título de Elio y reglas sobre Markdown, cuadernos e índices reconstruibles. Nightingale usa SQLite operativo local y PostgreSQL central. | Un colaborador o agente recibe instrucciones incompatibles. | T01 |
| Bloqueo de instalación reproducible | Falta lockfile JavaScript, versión declarada de pnpm y versión acordada de Node/Rust. `Cargo.lock` contiene solo el paquete raíz, sin Tauri. | Dos instalaciones pueden resolver dependencias distintas. | T02–T04 |
| Bloqueo de incorporación | `README.md` sigue siendo el de la plantilla; no explica instalación, comandos, requisitos nativos ni errores esperables. | Cada colaborador debe deducir cómo ejecutar el proyecto. | T11 |
| Falta de verificación | No hay CI, pruebas de comportamiento ni comando común de calidad. El escritorio no tiene evidencia de arranque en ninguno de los tres sistemas. | No se puede afirmar que un clon limpio funciona. | T08–T11 |
| Base incompleta | `src/App.tsx` y Rust solo implementan el saludo de Tauri. No hay Tailwind ni persistencia SQLite. | Existe una plantilla, pero aún no la base acordada en el PMD. | T05, T06, T08 |
| Desviación de seguridad | `src-tauri/tauri.conf.json` contiene `csp: null`; la capacidad principal permite `opener:default`. `.gitignore` no excluye `.env` ni bases locales en general. | La plantilla no aplica aún las reglas del DAT §17 y PMD §19/24. No se detectó una filtración. | T07 |
| Contradicción funcional futura | SRS §9 permite al paciente crear citas. PMD §4.3 y DAT §7.5 lo prohíben. SRS §7.2 propone elegir horarios; PMD §17.4 impide consultar/reservar la agenda. | La futura ruta Patient Hub no tiene una referencia uniforme. | T01, sin implementar Patient Hub |

### Comprobaciones y límites de la revisión

- Se leyeron completos `docs/master-plan.md`, `docs/srs.md` y `docs/technical-architecture.md`, además de los manifiestos, configuración y código de la plantilla.
- El repositorio estaba limpio al empezar. `tasks/` solo contenía `README.md`; no se reemplazó un plan abierto.
- Herramientas presentes en esta sesión: Node `24.18.0`, pnpm `11.17.0`, Cargo y rustc `1.99.1`, host Rust `stable-x86_64-pc-windows-msvc`. Son observaciones del equipo local, no versiones ya acordadas para el proyecto.
- `cargo metadata --no-deps --format-version 1 --offline`: falla con `no targets specified in the manifest`.
- `cargo metadata --manifest-path src-tauri/Cargo.toml --no-deps --format-version 1 --offline`: falla por el manifiesto raíz inválido.
- `pnpm build`: intenta preparar dependencias y falla al abrir el almacén local de pnpm. No prueba un fallo del código TypeScript.
- `pnpm --config.verify-deps-before-run=false run build`: evita esa preparación automática y falla porque falta `tsc`; no existe `node_modules/`. No se completó una instalación.
- `git check-ignore --no-index` solo reconoce `.env.local` entre `.env`, `.env.example`, `.env.development`, `.env.local`, `local.sqlite`, `local.sqlite-wal` y `local.db`. Los ejemplos inocuos deben seguir siendo versionables.
- No se comprobó la ejecución nativa, la instalación de todos los requisitos del sistema ni la compilación en macOS/Linux. No se declara ninguna de ellas aprobada.

### Qué ya sirve para empezar la base

El PMD separa dominio, aplicación y adaptadores; fija propiedad de datos, límites de componentes, errores, versiones y pruebas. También distingue estado de negocio y sincronización, exige una transacción para entidad/Outbox y conserva PostgreSQL como autoridad central. No hace falta rehacer esa arquitectura para arreglar la plantilla.

### Decisiones futuras que no bloquean esta base

- Lenguaje del servidor, autenticación cloud, proveedor de infraestructura y cola: cerrar antes de implementar esas rutas.
- Duración de servicios y reglas temporales: usar intervalos según PMD §15.3; cerrar políticas antes del motor de citas.
- DAT §11 no define todas las salidas de `reschedule_requested` ni la relación exacta con la solicitud separada. Cerrar antes de implementar las transiciones de citas.
- DAT §12.7 dibuja al ejecutor accediendo a PostgreSQL, aunque PMD §16.5 exige API interna para n8n. Corregir antes de la ruta de recordatorios.
- Cifrado, claves locales y tratamiento de un dispositivo revocado: cerrar antes de introducir datos reales. Los ensayos de esta base usan solo datos sintéticos.
- Retención del Change Log, recuperación por snapshot y dependencias entre varias operaciones offline necesitan contratos concretos antes de la sincronización.
- WhatsApp, políticas de recordatorios/lista de espera, ETL, objetivos de rendimiento y recuperación: cerrar al planificar sus funciones. No instalar servicios anticipados para esta base.

## Source of Truth

- Solicitud del usuario: analizar primero y preparar una base para colaboradores; tres sistemas operativos; servicios web fuera del trabajo inicial.
- `AGENTS.md`: plan aprobado antes de código, una tarea activa y preservación de trabajo ajeno. Sus reglas heredadas de Elio son un conflicto pendiente de T01.
- `tasks/README.md`: ID, estados, tareas y checkpoints.
- PMD §§3–8: precedencia, decisiones y estructura conceptual; §§11–13: primitivas, contratos y persistencia; §§19, 22–27: seguridad, interfaz, pruebas, configuración, construcción y documentación.
- DAT §§4, 7.1–7.2, 17–18: límites React/Rust, SQLite, capacidades y requisitos por sistema.
- SRS §§6, 9, 11, 16: alcance, reglas y contradicciones que se deben registrar/corregir.
- Implementación actual: `package.json`, `Cargo.toml`, `Cargo.lock`, `src/`, `src-tauri/`, `vite.config.ts`.

### Fuentes oficiales comprobadas durante la revisión

- Cargo: [manifiestos](https://doc.rust-lang.org/cargo/reference/manifest.html) y [workspace virtual](https://doc.rust-lang.org/cargo/reference/workspaces.html#virtual-workspace). Sustentan el diagnóstico y la propuesta de raíz Rust. Los perfiles de un workspace pertenecen al manifiesto raíz.
- Tauri 2: [requisitos por sistema](https://v2.tauri.app/start/prerequisites/), [CSP](https://v2.tauri.app/security/csp/) y [compilaciones en GitHub](https://v2.tauri.app/distribute/pipelines/github/). La guía de publicación solo sirve de referencia para los hosts nativos; este plan no autoriza publicar releases.
- pnpm: [ejecución de scripts](https://pnpm.io/cli/run) y [verificación previa de dependencias](https://pnpm.io/settings/build#verifydepsbeforerun). La repetición local con esa verificación desactivada confirmó que faltan dependencias.

Los manifiestos declaran Tauri 2, React `^19.1.0`, Vite `^8.0.16` y TypeScript `~6.0.3`; no hay versiones resueltas instaladas para esas bibliotecas. En implementación se deben consultar las páginas oficiales de las versiones que fije T04. No copiar opciones nuevas de la documentación actual sin comprobar su soporte en la versión elegida.

## Architecture Decisions

El usuario aprobó las propuestas al autorizar el inicio del plan y aceptó las decisiones de entorno de T02 en C01 el 2026-10-19.

1. Mantener el escritorio Tauri en las rutas actuales durante esta base. Registrar un mapa explícito: `src/` es la interfaz de escritorio y `src-tauri/` su proceso Rust. El PMD §8 presenta rutas conceptuales; no mover archivos solo para reproducir su árbol. Reservar la incorporación de otras aplicaciones/servicios para sus planes.
2. Convertir el manifiesto Rust raíz en workspace virtual con `src-tauri` como miembro. Resolver el ejemplo aislado `crates/main.rs` sin crear un segundo programa de negocio. Mantener un lockfile Rust real en raíz y situar allí el perfil de compilación.
3. Mantener pnpm, ya usado por los hooks Tauri; fijar herramientas y generar el lockfile. No cambiar de gestor ni actualizar bibliotecas por rutina.
4. Conservar límites React → cliente IPC tipado → comando Rust específico. El diagnóstico solo mostrará datos reales de esta instalación. No simulará sesión, sincronización o conexión cloud.
5. Añadir SQLite solo para apertura, migración y reapertura de una base vacía en el directorio de datos de la aplicación. Elegir el adaptador mínimo tras comprobar compatibilidad con el cifrado futuro. No crear tablas de pacientes, citas u Outbox sin su caso de uso.
6. No crear carpetas vacías, una librería global de modelos, un sistema de diseño ni interfaces para todos los servicios futuros. Documentar dónde crecerá cada parte.
7. Probar migraciones y recuperación de errores con bases temporales aisladas. Usar herramientas ya disponibles para tipos, compilación y Rust; no añadir una suite frontend que solo compruebe texto de la plantilla.
8. Ejecutar los controles locales y probar la apertura en Windows, macOS y Fedora 44. El usuario retiró los jobs de CI el 2026-10-21; no exigir resultados futuros de esos jobs para cerrar esta base. Se conservan las versiones y arquitecturas acordadas y sus pruebas manuales.

## Dependency Order

```text
T01 → T02 → C01
T03 → T04 → C02
T05 → T06 → T07 → C03
T08 → T09 → C04
T10 → T11 → C05
```

Trabajar en una tarea cada vez. Cada checkpoint requiere evidencia y revisión del usuario antes de la siguiente fase. Cuando una comprobación falle por acceso o red, conservar el error y distinguirlo de un defecto del proyecto.

## Task List

### Phase 1: Reglas y alcance de la base

- [x] T01: Corregir las instrucciones contradictorias.
- [x] T02: Acordar el entorno de desarrollo.

### C01: Referencias y entorno revisados

- [x] Las reglas describen Nightingale y las tres plataformas; el mapa de rutas y las versiones elegidas tienen aprobación.
- [x] No hay cambios funcionales de producto ni elección de servicios web.
- [x] Revisión del usuario completa.

### Phase 2: Compilación reproducible

- [x] T03: Reparar el workspace Rust.
- [x] T04: Fijar la instalación JavaScript.

### C02: Plantilla compilable

- [x] Cargo carga el proyecto; los lockfiles corresponden a los manifiestos.
- [x] La interfaz compila y el escritorio abre en el host disponible. La prueba de los demás hosts queda pendiente hasta C05.
- [x] Revisión del usuario completa.

### Phase 3: Esqueleto de Nightingale

- [x] T05: Integrar Tailwind para estilos básicos.
- [x] T06: Mostrar un diagnóstico local real.
- [x] T07: Aplicar los límites de seguridad de la base.

### C03: React y Rust conectados

- [x] La app muestra su diagnóstico, conserva foco visible y comunica fallos sin trazas internas.
- [x] La CSP permite solo los recursos necesarios y los archivos locales sensibles quedan excluidos de Git.
- [x] Revisión del usuario completa.

### Phase 4: Persistencia local comprobable

- [x] T08: Inicializar SQLite con migraciones.
- [x] T09: Unificar los controles de calidad.

### C04: Base local estable

- [x] Las pruebas cubren creación, reapertura, fallo y versión de esquema no compatible sin modificar una base existente.
- [x] El comando de calidad ejecuta comprobaciones reales y falla ante errores.
- [x] Revisión del usuario completa.

### Phase 5: Incorporación del equipo

- [x] T10: Comprobar los tres sistemas en CI. Retirada por el usuario el 2026-10-21; se cierra por retirada, no por una matriz aprobada.
- [x] T11: Documentar la incorporación de colaboradores. Cierre aceptado por el usuario sin las comprobaciones manuales restantes.

### C05: Base lista para colaboradores

- [x] Instalación y controles respaldados por las pruebas Windows y la CI histórica de los cuatro entornos; el usuario acepta esta evidencia para el cierre.
- [x] Apertura y diagnóstico comprobados en Windows. El usuario acepta el cierre sin pruebas manuales Fedora/macOS; no se declaran realizadas.
- [x] El usuario acepta la base sin revisión de otro colaborador. La guía existe y se probó localmente; no se afirma una revisión independiente.

## Risks and Mitigations

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Corregir referencias cambia sin querer el alcance del producto | Alto | T01 solo alinea decisiones ya asentadas; registrar cualquier nueva regla y pedir revisión. |
| El mapa de rutas actual no satisface la organización que espera el equipo | Medio | Resolver en C01 antes de cambios de estructura. No hacer una mudanza masiva implícita. |
| Dependencias declaradas no resuelven juntas | Alto | Comprobar versiones y requisitos oficiales en T04; presentar cualquier cambio necesario de versión. |
| pnpm falla por su almacén o por red | Medio | Separar el problema local de la compilación; no cambiar código para ocultarlo. |
| El adaptador SQLite impide el cifrado futuro | Alto | Revisar su compatibilidad antes de elegirlo y evitar datos reales. |
| Solo se dispone de Windows en esta sesión | Alto | Pruebas locales y revisión manual en los otros equipos; no afirmar compatibilidad por inferencia. |
| Se confunde una base funcional con citas o sincronización terminadas | Alto | Diagnóstico explícito de funciones disponibles; planes posteriores por ruta. |

## Open Questions

- Resuelto en C01: Windows 11 x64, macOS 15 ARM/Intel y Ubuntu 24.04 x64; el usuario corrigió Linux a Fedora 44 el 2026-10-19. Mapa de rutas actual y herramientas del [ADR de la base de escritorio](../../docs/decisions/desktop-foundation.md): Node 24.18.0, pnpm 11.17.0 y Rust/Cargo 1.99.1.
- En T08: seleccionar el adaptador SQLite mínimo y dejar constancia de su relación con el cifrado futuro antes de instalarlo.
- No se requiere elegir backend, nube o WhatsApp para aprobar esta base.

## Execution Evidence

### C01: aceptado el 2026-10-19

- T01 completada: reglas de Nightingale, SRS 0.3 y pendientes futuros registrados sin modificar código de producto.
- T02 preparada y comprobada: ADR con rutas, herramientas, requisitos y cuatro hosts propuestos. Se verificaron fuentes oficiales, enlaces y coherencia con los manifiestos; no se afirma que las dependencias completas compilen.
- `git diff --check` pasa; solo se han cambiado archivos Markdown.
- No hay remoto Git configurado en esta revisión; los resultados de CI no estarán disponibles hasta configurarlo y ejecutar la matriz.
- El usuario respondió «Puedes continuar, por favor» a la solicitud de aceptar C01. Se registra la aceptación y se inicia T03. Los puntos anteriores describen las comprobaciones de T01–T02, antes de los cambios de compilación.

### C02: aceptado el 2026-10-19

- T03 completada: workspace nativo con el único miembro `src-tauri`, perfil release en raíz y Rust 1.99.1 fijado. Ambos comandos de metadatos, Cargo check bloqueado y formato pasan. El lockfile real incluye Tauri 2.12.4.
- T04 completada: Node/pnpm declarados, lockfile JavaScript generado e instalación fija repetida sin cambios. Compilación web y arranque nativo Windows comprobados; el saludo de React recibe la respuesta Rust y la ventana cierra correctamente.
- Se corrigió un bloqueo de arranque observado: Vite ahora ignora `target/` para no vigilar los archivos que Cargo compila en el workspace.
- Los detalles, comandos, hashes y fuentes oficiales quedan en la evidencia de T03/T04. La apertura de macOS y Linux y la matriz CI siguen pendientes.
- El usuario respondió «Lo veo todo bien, podemos continuar» y aceptó C02. Se inicia T05 dentro del grupo T05–T07.

### Estado tras C04 y preparación de T10–T11 (2026-10-19)

C03 y C04 aceptados por el usuario. T08–T09 verificados: SQLite transaccional con siete pruebas, apertura/reapertura Windows, control de calidad y detección de fallo real. T10 tiene el workflow de cuatro hosts y T11 las guías y una prueba de clon limpio Windows con perfil de datos nuevo. No se marcan T10, T11 ni C05 completos: faltan resultados remotos, aperturas macOS/Linux y revisión de otro colaborador. El usuario dejó la configuración del remoto pendiente a su cargo. La evidencia y la matriz pendiente están en todo.md. No hay trabajo de servicios web en este cambio.

### Corrección del entorno Linux por el usuario (2026-10-19)

El equipo usa Fedora 44, no Ubuntu. Se adapta la parte aún abierta de T10/T11: paquetes dnf para desarrollo y un job Fedora 44 en contenedor sobre un host Ubuntu de GitHub. El host no define la distribución de las bibliotecas de compilación. Se mantienen Windows y las dos arquitecturas macOS; el usuario no dispone ahora de equipos para las validaciones pendientes. No se cambia el alcance funcional, no se inventan resultados y CI sigue pendiente a cargo del propietario. La corrección actualiza el ADR y las guías; la evidencia previa de Ubuntu describe solo la propuesta anterior.

### Retirada de CI por el usuario (2026-10-21)

El usuario pidió «Es mejor eliminar estos jobs, por favor» después de revisar la primera ejecución remota. Se elimina el workflow del repositorio y sus referencias activas de las guías. T10 se cierra por retirada explícita del requisito; su evidencia anterior queda como historia, sin convertir el fallo Fedora en éxito. T11 y C05 conservan la validación manual en macOS/Fedora, la revisión por otro colaborador y la aceptación final. Los controles locales y el código de producto no cambian.

### Cierre aceptado (2026-10-21)

El usuario pidió «Por favor, marca esas como listas» respecto a las comprobaciones externas restantes. El plan se cierra por esa aceptación explícita, sin afirmar que se ejecutaron las aperturas Fedora/macOS o una revisión independiente. Se conserva la evidencia técnica previa y se distingue de los puntos aceptados sin verificar. Las menciones históricas a bloqueos ya no definen el estado actual del plan.
