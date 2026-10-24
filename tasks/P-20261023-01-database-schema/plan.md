# P-20261023-01-database-schema: Documentación del esquema de base de datos

**Status:** Completed  
**Created:** 2026-10-23  
**Updated:** 2026-10-23  
**Task list:** [todo.md](todo.md)

## Overview

Preparar una primera versión revisable del esquema físico de PostgreSQL para cuentas, usuarios, profesionales, pacientes, servicios, agenda, citas e historial. La fuente de este diseño será [docs/database-schema.md](../../docs/database-schema.md). No se referenciarán otros planes de modelos de datos.

El usuario aprobó este plan al pedir «PLEASE IMPLEMENT THIS PLAN» y reproducir su contenido en la conversación del 2026-10-23. Esa aprobación permite realizar las tareas en orden; no acepta por anticipado los checkpoints ni las propuestas físicas del documento.

La entrega describirá propósito y propietario de tablas, columnas, tipos, nulabilidad, valores por defecto, claves, restricciones, índices, cardinalidad, conservación y ejemplos ficticios. Incluirá un diagrama Mermaid general y tres de detalle, con nombres iguales al diccionario.

Quedan fuera SQL, migraciones, modelos de aplicación, contratos HTTP, SQLite, sincronización, auditoría técnica, comunicaciones, lista de espera y módulos clínicos, de inventario, facturación o documentos médicos. No se ejecutará una base de datos para acreditar esta entrega documental.

## Source of Truth

- Plan reproducido y aprobado por el usuario y acuerdos de esta conversación del 2026-10-23.
- [Reglas de trabajo](../../AGENTS.md) y [nombres de planes y tareas](../README.md).
- [Plan maestro](../../docs/master-plan.md), §§3, 6–7, 11, 13, 19 y 27, en lo que no sustituyan los acuerdos nuevos.
- [Arquitectura técnica](../../docs/technical-architecture.md), §§7, 13 y 17; [SRS](../../docs/srs.md), §§8–10; [decisión de tres proyectos](../../docs/decisions/transactional-sync-three-projects.md).
- Documentación oficial de PostgreSQL enlazada desde el documento del esquema para cada mecanismo usado.

## Architecture Decisions

- Una única fuente editable del esquema en este repositorio por elección expresa del usuario. La ubicación de la documentación no traslada la propiedad del PostgreSQL operativo: corresponde a Nightingale API.
- El primer uso es el consultorio independiente. El modelo admitirá clínicas y pertenencias a varias cuentas; no incorpora ahora gestión clínica avanzada. Esta instrucción sustituye para este diseño el límite anterior de un profesional por cuenta en todos los casos.
- Se documentan relaciones y datos; no se añaden casos de uso Rust, comandos Tauri, clientes HTTP ni servidores. La operación local y los últimos permisos válidos sin red conservan sus reglas vigentes; sus esquemas y contratos quedan fuera de esta entrega.
- Se separan identidad global de paciente, ficha por cuenta y contactos por cuenta, sin conceder acceso entre consultas por coincidencia de identidad. El menor sin documento podrá citarse con nombre, fecha de nacimiento y contacto responsable; completar su documento no cambia su identidad.
- Se distinguen acuerdos del usuario, propuestas físicas y pendientes. Los detalles físicos nuevos requieren revisión en el checkpoint correspondiente.
- No se actualizan como efecto secundario el plan maestro, la SRS, la arquitectura ni otros repositorios. El documento identificará las reglas anteriores que reemplazan las instrucciones actuales.
- Trabajo secuencial, con una sola tarea activa y sin delegación. No hay cambios de contratos compartidos ni tareas paralelas.

## Dependency Order

T01 → T02 → C01 → T03 → T04 → C02 → T05 → T06 → C03.

La evidencia de cada tarea se registra después de comprobarla. Se detiene el trabajo en cada checkpoint hasta la revisión del usuario.

## Task List

### Phase 1: Convenciones y cuentas

- [x] T01: Fijar convenciones.
- [x] T02: Cuentas y profesionales.

### C01: Cuentas, usuarios y profesionales

- [x] Convenciones, tablas y relaciones comprobadas; diagrama de este grupo renderizado.
- [x] Revisión del usuario completada antes de T03.

**Aceptación (2026-10-23):** el usuario respondió «Puedes continuar, por favor» a la entrega de C01. Se acepta esa etapa y se habilitan T03–T04; C02 conserva su revisión propia.

### Phase 2: Pacientes y agenda

- [x] T03: Pacientes y contactos.
- [x] T04: Servicios y agenda.

### C02: Pacientes, contactos, servicios y agenda

- [x] Reglas de borrador, menor sin documento, aislamiento y disponibilidad representadas.
- [x] Revisión del usuario completada antes de T05.

**Aceptación (2026-10-23):** el usuario respondió «Sí, por favor continua» a la entrega de C02. Se habilitan T05 y T06; C03 conserva su revisión final.

### Phase 3: Citas y revisión del conjunto

- [x] T05: Citas e historial.
- [x] T06: Diagramas y revisión completa.

### C03: Esquema completo

- [x] Tablas, referencias, restricciones, índices, ejemplos y cuatro diagramas coherentes y comprobados.
- [x] Revisión final del usuario completada.

**Aceptación (2026-10-23):** el usuario respondió «Confirmado» a la solicitud de aceptar C03 y cerrar esta versión documental. Las seis tareas y los tres checkpoints quedan completos; no se autoriza por ello implementación de tablas o modelos.

## Risks and Mitigations

| Riesgo | Impacto | Tratamiento |
|---|---|---|
| Confundir propuestas físicas con acuerdos aprobados | Alto | Identificar su estado y conservar revisión por checkpoint. |
| Conceder acceso por una identidad global o un parentesco | Alto | Separar identidad, pertenencia, contacto y permisos; comprobar escenarios entre cuentas. |
| Presentar una FK o CHECK como prueba de todas las reglas entre tablas | Alto | Explicar qué garantiza cada restricción y qué necesita integridad transaccional adicional. |
| Duplicar personas o unir pacientes por coincidencias débiles | Alto | Identificadores estables y revisión explícita de coincidencias; no derivar identidad del representante. |
| Documentar clínicas como funcionalidad ya construida | Medio | Distinguir capacidad del modelo de alcance del primer uso y de implementación. |
| Atribuir pruebas de PostgreSQL a comprobaciones del texto | Alto | Evidencia documental separada; sin pruebas SQL, de concurrencia o de aplicación. |
| Reemplazar cambios previos | Alto | Comparar hashes de los archivos existentes y limitar escrituras a la entrega de este plan. |

## Open Questions

C01, C02 y C03 están aceptados. La versión documental 0.3 queda cerrada, con las seis tareas terminadas. Formatos exactos de documento/teléfono, procedimientos de comprobación, reglas legales por edad y otros límites enumerados en §8.3 conservan su estado pendiente para el trabajo que corresponda; la aceptación del esquema no inventa esas reglas ni declara pruebas de una base ejecutada.

## Verification

Para cada tarea: revisar enlaces locales, bloques Markdown, referencias a columnas/claves y límites del alcance; ejecutar `git diff --check` y comprobar también los archivos nuevos todavía no seguidos por Git. Renderizar y revisar los diagramas correspondientes. No procede compilar la aplicación ni ejecutar pruebas de producto por esta entrega.

La línea base del 2026-10-23 contiene 118 archivos existentes, incluidos cambios documentales previos del usuario. Se conservarán sus hashes. No se han creado ni aceptado checkpoints por escribir este plan.

## Execution Evidence

2026-10-23: T01 y T02 terminadas. La entrega parcial contiene cinco tablas, 34 columnas y siete FK. Se comprobaron los tipos y destinos de las referencias y la coincidencia exacta de columnas/tipos entre diccionario y diagrama. Mermaid 11.12.0 analizó y renderizó el diagrama sin errores; PNG y SVG de revisión en `target/database-schema-review/`, fuera de la fuente documental y de Git. Se inspeccionó visualmente el PNG de 1641×1050 píxeles. Se verificaron enlaces locales, Markdown y espacios; `git diff --check` terminó con código 0.

Los 118 archivos de la línea base conservan sus SHA-256. Las únicas altas de fuente son el documento del esquema y los dos archivos de este plan. No se modifican documentos previos, código, dependencias u otros repositorios. No se ejecutó PostgreSQL, SQL, migraciones ni pruebas de aplicación. C01 queda pendiente del usuario; T03–T06 no han comenzado.

### C01 aceptado y entrega T03–T04 · 2026-10-23

El usuario respondió «Puedes continuar, por favor». Se registró C01 y se ejecutaron T03 y después T04, con sus comprobaciones separadas. El documento 0.2 suma 15 tablas, 122 columnas, 22 FK y tres diagramas de detalle; el tercero se ampliará con Citas en T05 y el general corresponde a T06. Se contrastaron todos los atributos y tipos con el diccionario y se verificaron los destinos PK/UQ de las FK, incluidos los ámbitos por cuenta.

Mermaid 11.12.0 analizó y renderizó los tres diagramas sin errores. Se inspeccionaron las imágenes de pacientes y agenda, con entidades y etiquetas completas; SVG/PNG siguen en `target/database-schema-review/`. Revisión documental de tres Markdown y 13 enlaces locales sin incidencias. `git diff --check` pasa con la configuración del repositorio; comprobación directa adicional de archivos nuevos, espacios y bloques.

La comparación de la línea base de esta etapa confirma 118 archivos ajenos intactos y cambios limitados al documento y los dos archivos de este plan. El diccionario, referencias y diagrama de §§3.1–3.9 aceptados en C01 mantienen exactamente su contenido. No se ejecutó SQL ni una base PostgreSQL y no se cambiaron modelos o migraciones. C02 está pendiente del usuario; T05–T06 siguen sin comenzar.

### C02 aceptado y entrega T05–T06 · 2026-10-23

El usuario respondió «Sí, por favor continua». Se registró C02 y se ejecutaron T05 y T06 en secuencia. El documento 0.3 contiene la primera entrega completa: 17 tablas, 161 columnas, 30 FK, tres diagramas de detalle y uno general. Los datos actuales de cita y su historial conservan intervalo, confirmación, valores acordados y actor funcional; la exclusión por profesional abarca sus cuentas y se distingue de la coordinación necesaria con Agenda.

Comprobaciones: tres Markdown, 19 enlaces locales —seis de navegación interna—, 161 filas de diccionario completas y sin columnas repetidas; referencias F01–F30 consecutivas, destinos PK/UQ y tipos compatibles; los atributos de los detalles coinciden con los diccionarios. Mermaid 11.12.0 analizó y renderizó los cuatro diagramas sin errores. Se inspeccionaron los detalles y la vista general; esta última se simplificó para lectura y remite a los detalles para los vínculos secundarios. Se revisaron los 17 recorridos de §8.2 y el ejemplo de historia de versiones 1–3. No son pruebas SQL ni de concurrencia ejecutadas.

Las 15 tablas y 122 filas de diccionario aceptadas antes de esta etapa conservan exactamente su definición; se añaden solo las dos tablas de Citas. Se verificaron 118 archivos ajenos con SHA-256, sin cambios; el alcance de edición sigue limitado al documento y los dos archivos de este plan. Las imágenes SVG/PNG de revisión permanecen en `target/database-schema-review/`, fuera de la fuente y de Git. `git diff --check` pasa y el control directo cubre los documentos nuevos todavía no seguidos.

T01–T06 están Done. C03 queda pendiente del usuario y el plan sigue In progress; no se registra aceptación final por superar comprobaciones documentales. No se ha ejecutado PostgreSQL, SQL, migraciones o modelos de aplicación, ni se han editado otros repositorios o especificaciones previas.

### Cierre documental aceptado · 2026-10-23

El usuario confirmó C03 con «Confirmado». Se marcan su revisión final y el plan como Completed. El esquema conserva la versión 0.3 y sus definiciones; solo se actualizan los estados y se registra esta aceptación. Las notas anteriores de checkpoints pendientes describen el estado histórico de cada entrega. Se conserva la evidencia documental y sus límites: ninguna comprobación se convierte en prueba SQL, de concurrencia o de aplicación. No se inicia implementación ni se modifica el esquema aceptado durante este cierre.
