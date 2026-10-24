# Task List: P-20261023-01-database-schema

**Plan:** [plan.md](plan.md)  
**Updated:** 2026-10-23

El usuario aprobó ejecutar el plan en la conversación. Se trabaja una tarea cada vez. La aprobación del plan no sustituye las revisiones C01–C03.

## T01: Fijar convenciones

**Status:** Done

**Description:** Definir alcance, vocabulario, nombres, identificadores, tiempo, estados y distinción entre datos globales y por cuenta en el documento único del esquema.

**Acceptance criteria:**
- [x] Distingue acuerdos del usuario, propuestas físicas y pendientes, con alcance PostgreSQL sin SQL.
- [x] Define nombres, tipos base, obligatoriedad, valores por defecto, claves e índices.
- [x] Separa datos globales y por cuenta y registra los cambios de alcance acordados sin modificar otras especificaciones.

**Verification:**
- [x] Focused tests: revisar las convenciones contra los acuerdos y fuentes oficiales enlazadas.
- [x] Build or broader check: enlaces, bloques Markdown, revisión directa de espacios y `git diff --check`; compilación no aplicable.
- [x] Manual check: leer un ejemplo de identidad y pertenencia sin deducir acceso por su existencia.

**Evidence (2026-10-23):** se comprobaron tres Markdown y 13 enlaces locales sin incidencias, bloques cerrados y espacios; `git diff --check` terminó con código 0. Convenciones contrastadas con las páginas oficiales PostgreSQL 18 de UUID, tiempo, texto, tipos numéricos, claves, índices parciales y restricciones diferidas. El ejemplo distingue la identidad de Ana de sus pertenencias. Los acuerdos nuevos se registran solo en la entrega; no se han escrito tablas, SQL ni modelos de producto.

**Dependencies:** None  
**Files likely touched:** `docs/database-schema.md`, `plan.md`, `todo.md` de este plan.  
**Estimated scope:** S

## T02: Cuentas y profesionales

**Status:** Done

**Description:** Describir físicamente cuentas, usuarios, perfiles profesionales y pertenencias, con su diagrama de detalle y ejemplos.

**Acceptance criteria:**
- [x] Cada tabla tiene propietario, columnas/tipos/nulabilidad/defaults, claves, restricciones, índices y conservación.
- [x] Un profesional tiene usuario; las pertenencias admiten varias cuentas sin duplicar a la persona y cada cuenta representa un lugar.
- [x] Cardinalidades y diagrama coinciden; las reglas entre tablas distinguen garantías de claves y controles adicionales.

**Verification:**
- [x] Focused tests: comprobar PK/FK y casos de profesional independiente, profesional en clínicas y asistente en varias cuentas por lectura del esquema.
- [x] Build or broader check: enlaces, referencias, render de Mermaid, espacios y `git diff --check`; sin ejecución de PostgreSQL.
- [x] Manual check: inspeccionar el diagrama renderizado y los ejemplos antes de presentar C01.

**Evidence (2026-10-23):** cinco tablas y 34 columnas; siete FK con campos existentes, tipos compatibles y destino PK/UQ documentado. Un verificador de biblioteca estándar contrastó atributos Mermaid y diccionario sin incidencias. Se revisaron los casos de pertenencia múltiple, perfil vinculado al usuario ajeno, propietaria no médica y baja en una sola clínica. Las condiciones de cuenta independiente figuran como integridad diferida propuesta, no como una garantía probada por FK.

Mermaid 11.12.0 analizó y renderizó el diagrama sin errores de página; se generaron SVG y PNG de revisión en `target/database-schema-review/`. La imagen de 1641×1050 se inspeccionó: cinco entidades y enlaces legibles, sin texto cortado. La primera ruta de navegador no existía y el arranque de Chromium instalado falló bajo el entorno aislado; se renderizó con ese Chromium mediante el acceso permitido, sin instalar paquetes ni navegadores. No se ejecutó la aplicación.

Comprobaciones: tres Markdown, 13 enlaces locales, bloques y espacios; `git diff --check` correcto. Se compararon los 118 archivos existentes con sus SHA-256 iniciales: sin cambios. Solo se añaden `docs/database-schema.md` y los dos archivos de este plan. Fuentes oficiales específicas figuran en §4 del documento. No se ejecutó PostgreSQL ni se afirma prueba de aislamiento o concurrencia.

**Dependencies:** T01  
**Files likely touched:** `docs/database-schema.md`, `plan.md`, `todo.md` de este plan.  
**Estimated scope:** M

## C01: Después de T01–T02

- [x] Convenciones, diccionario de cuentas/profesionales y diagrama comprobados.
- [x] Propuestas y límites disponibles para revisión del usuario.
- [x] Revisión del usuario completada; habilita T03.

**Estado al entregar:** pendiente solo de revisión del usuario. Revisar especialmente las propuestas enumeradas en §3.10 del esquema. T03–T06 siguen en Todo; no se continúa por una aprobación implícita ni por haber pasado las comprobaciones documentales.

**Aceptación posterior (2026-10-23):** el usuario respondió «Puedes continuar, por favor». C01 queda aceptado y se inicia T03. El estado al entregar de la nota anterior es histórico; C02 y C03 no quedan aceptados por esta respuesta.

## T03: Pacientes y contactos

**Status:** Done

**Description:** Definir identidad, documentos venezolanos, fichas por cuenta, personas de contacto y vínculos, con diagrama de detalle.

**Acceptance criteria:**
- [x] Todas las tablas tienen diccionario físico, referencias, restricciones, índices y conservación.
- [x] Borrador/completitud y actividad son independientes; admite menor sin documento con los datos mínimos acordados.
- [x] La identidad no abre datos de otra cuenta; parentesco no acredita tutela ni acceso; coincidencias débiles no unen pacientes automáticamente.

**Verification:**
- [x] Focused tests: revisión documental de mismo paciente en dos cuentas, hermanos con un contacto, menor que obtiene documento y ficha incompleta.
- [x] Build or broader check: coherencia con tablas de T02, enlaces, Mermaid renderizado, espacios y `git diff --check`.
- [x] Manual check: recorrer identificación, ficha y contacto sin usar el documento del adulto como identidad del menor.

**Evidence (2026-10-23):** cinco tablas nuevas con 45 columnas y ocho FK; el documento suma diez tablas, 79 columnas y 15 FK. Referencias contrastadas con destinos PK/UQ y tipos compatibles, atributos Mermaid iguales al diccionario, 13 enlaces locales y espacios sin incidencias. `git diff --check` pasa con la configuración del repositorio. Una comprobación previa con `core.autocrlf=false` produjo avisos por los CRLF de archivos preexistentes; no se cambiaron esos archivos ni la configuración para corregirlos. Mermaid 11.12.0 analizó y renderizó los dos diagramas sin errores; el PNG de pacientes se inspeccionó visualmente. Se recorrieron los siete casos ficticios de §4.9, incluidos ficha por cuenta, hermanos, documento posterior y coincidencia no confirmada. Son comprobaciones documentales; no hay SQL o PostgreSQL ejecutado ni verificación legal de tutela. Las propuestas de §4.9 esperan C02.

**Dependencies:** T02 y C01 aceptado  
**Files likely touched:** `docs/database-schema.md`, `plan.md`, `todo.md` de este plan.  
**Estimated scope:** M

## T04: Servicios y agenda

**Status:** Done

**Description:** Definir servicios por cuenta, relación con profesionales, horarios habituales, excepciones y bloqueos.

**Acceptance criteria:**
- [x] Diccionario completo y restricciones coherentes con cuenta y profesional.
- [x] La duración por defecto no modifica citas previas; horarios y bloqueos tienen alcance y tiempo definidos.
- [x] Los calendarios derivan de datos existentes y la ocupación del profesional abarca sus cuentas sin revelar detalles ajenos.

**Verification:**
- [x] Focused tests: revisión de cambio de duración, horario habitual, excepción, vacaciones y bloqueo frente a una cita existente.
- [x] Build or broader check: referencias a cuentas/profesionales, enlaces, espacios y `git diff --check`; sin pruebas de aplicación.
- [x] Manual check: representar la vista por cuenta y la del profesional sin duplicar reservas.

**Evidence (2026-10-23):** cinco tablas nuevas, 43 columnas y siete FK. El documento completo de esta etapa contiene 15 tablas, 122 columnas y 22 FK, todas con destinos y tipos compatibles. Los tres diagramas corresponden a sus diccionarios; `time` solo abrevia `time without time zone` como indica el texto. Revisión de los nueve casos ficticios de §5.9; las reservas y su integridad frente a cambios de agenda se declaran pendientes de T05, no implementadas.

Mermaid 11.12.0 analizó y renderizó los tres diagramas sin errores de página; se inspeccionaron pacientes y agenda con texto completo y relaciones legibles. Salidas en `target/database-schema-review/`. Tres Markdown, 13 enlaces locales y comprobación directa de espacios/bloques sin incidencias. `git diff --check` termina con código 0. Las páginas oficiales PostgreSQL 18 confirman tipos de fecha/hora, claves, nulabilidad, unicidad parcial y tratamiento de nulos iguales en la UQ semanal. Son fuentes de diseño, no pruebas de una base ejecutada.

Control de alcance: 118 archivos ajenos conservan sus hashes; solo cambian el documento del esquema y los dos archivos de este plan. Las definiciones de cuentas/profesionales y su diagrama aceptados en C01 se conservaron. No se añadieron dependencias, SQL, migraciones ni código de producto. C02 pendiente; T05–T06 sin iniciar.

**Dependencies:** T03  
**Files likely touched:** `docs/database-schema.md`, `plan.md`, `todo.md` de este plan.  
**Estimated scope:** M

## C02: Después de T03–T04

- [x] Pacientes, contactos, servicios y agenda comprobados con los casos acordados.
- [x] Propuestas y reglas aún pendientes distinguidas.
- [x] Revisión del usuario completada; habilita T05.

**Estado al entregar:** T03–T04 terminadas; revisar propuestas de §§4.9 y 5.9. No se interpreta la aceptación anterior de C01 como aceptación de este checkpoint. El diagrama general, Citas e historial siguen pendientes de T05–T06 después de esta revisión.

**Aceptación posterior (2026-10-23):** el usuario respondió «Sí, por favor continua». C02 queda aceptado y se inicia T05. Las notas de entrega anteriores son históricas; C03 sigue pendiente.

## T05: Citas e historial

**Status:** Done

**Description:** Definir citas, sus referencias e historial funcional, conservación y mecanismos de integridad temporal.

**Acceptance criteria:**
- [x] Cada cita vincula cuenta, paciente, profesional y servicio compatibles y conserva intervalo y estado.
- [x] Describe el mecanismo PostgreSQL contra solapamientos entre cuentas y sus límites ante bloqueos, sin SQL.
- [x] Citas pasadas siguen en su tabla; historial conserva cambios, responsable y tiempo, incluida invalidación de confirmación al reprogramar.

**Verification:**
- [x] Focused tests: revisión documental de reserva simultánea, reprogramación, cancelación e historial, con citas de distintas cuentas.
- [x] Build or broader check: referencias y tipos, fuentes oficiales para integridad temporal, Mermaid renderizado y `git diff --check`.
- [x] Manual check: reconstruir el cambio de horario y comprobar que el servicio editado no cambia el intervalo anterior.

**Evidence (2026-10-23):** dos tablas nuevas, 39 columnas y ocho FK. El documento suma 17 tablas, 161 columnas y 30 FK con destinos PK/UQ y tipos compatibles. Tres diagramas de detalle analizados y renderizados por Mermaid 11.12.0 sin errores; se inspeccionó el diagrama ampliado de servicios/agenda/citas. Comparación de todos los atributos con los diccionarios, enlaces, bloques y espacios sin incidencias; `git diff --check` termina con código 0.

Se recorrieron los doce casos de §6.7 y el ejemplo explícito de versiones 1–3: creación, confirmación y reprogramación; catálogo 30→20 conserva intervalo de 45 y valor acordado de 30; confirmación queda nula al cambiar horario. Se contrastaron exclusión global por profesional, actores sin usuario de paciente, correspondencia de historia y ciclos de FK diferidas con las fuentes oficiales enlazadas. El control frente a bloqueos requiere coordinación adicional; la exclusión no se presenta como prueba de todas las reglas de Agenda. No se ejecutó SQL ni PostgreSQL; la concurrencia sigue sin prueba de ejecución. Las propuestas nuevas esperan C03.

**Dependencies:** T04 y C02 aceptado  
**Files likely touched:** `docs/database-schema.md`, `plan.md`, `todo.md` de este plan.  
**Estimated scope:** M

## T06: Diagramas y revisión completa

**Status:** Done

**Description:** Completar el diagrama general, revisar los tres diagramas de detalle y contrastar el diccionario y ejemplos del conjunto.

**Acceptance criteria:**
- [x] Cuatro diagramas renderizados, legibles y coincidentes con tablas, campos clave y cardinalidades.
- [x] Cada tabla tiene definición física, claves, índices y conservación; no hay referencias sin destino ni SQL o modelos de aplicación.
- [x] Todos los escenarios acordados se explican; las comprobaciones documentales no se presentan como pruebas de una base ejecutada.

**Verification:**
- [x] Focused tests: matriz de casos ficticios y revisión cruzada de diagramas, columnas, claves y restricciones.
- [x] Build or broader check: enlaces, espacios, `git diff --check` y comparación de archivos protegidos; sin compilación ni pruebas de producto.
- [x] Manual check: inspección visual de los cuatro diagramas y lectura completa antes de C03.

**Evidence (2026-10-23):** 17 tablas con 161 columnas; 30 FK numeradas sin huecos o duplicados, destinos PK/UQ y tipos compatibles. Atributos de todos los detalles iguales al diccionario, con la abreviatura `time` documentada. Seis enlaces internos resueltos y 19 enlaces locales entre los tres Markdown; filas con tipo, obligatoriedad y valor por defecto, sin duplicados. Se actualizaron referencias futuras de T05 hacia las secciones ya terminadas, sin cambiar las 122 columnas de las 15 tablas previamente aceptadas.

Mermaid 11.12.0 analizó y renderizó los cuatro diagramas sin errores de página. Se inspeccionaron las imágenes de detalle y la vista general simplificada. Los SVG/PNG se conservan en `target/database-schema-review/`; la vista general es de relaciones principales, con referencias completas en F01–F30. Se recorrieron los 17 escenarios de §8.2, los doce casos de Citas y el ejemplo de tres versiones. Se comprobaron conservación, cambios de catálogo, actores sin usuario de paciente y límites de exclusión/coordinación. No se ha ejecutado una base PostgreSQL ni se afirma evidencia de concurrencia real.

Control de alcance: 118 archivos ajenos intactos según SHA-256; solo cambian el esquema y los dos archivos de este plan. No se añadieron archivos de código, SQL, migraciones o dependencias. `git diff --check` y la comprobación directa de los documentos nuevos pasan. La revisión C03 aún requiere aceptación del usuario.

**Dependencies:** T05  
**Files likely touched:** `docs/database-schema.md`, `plan.md`, `todo.md` de este plan.  
**Estimated scope:** M

## C03: Después de T05–T06

- [x] Diccionario completo y cuatro diagramas comprobados.
- [x] Evidencia y límites de verificación registrados.
- [x] Revisión final del usuario completada; habilita cierre documental, no implementación.

**Estado al entregar:** documento 0.3 completo y tareas T01–T06 terminadas. C01/C02 conservan su aceptación expresa; C03 queda pendiente. Revisar especialmente §§6.7 y 8.3–8.4, sin convertir las comprobaciones del texto y Mermaid en resultados de una base ejecutada.

**Aceptación posterior (2026-10-23):** el usuario respondió «Confirmado» a la solicitud de aceptar C03 y cerrar la versión documental. C03 queda aceptado y el plan Completed. La nota anterior conserva el estado histórico de entrega; las seis tareas y los tres checkpoints están completos. El cierre cambia solo estados y constancia de aceptación, no tablas, columnas, restricciones o diagramas. Los pendientes y límites de §8.3 permanecen; no se ha ejecutado una base de datos ni se autoriza implementación por este cierre.
