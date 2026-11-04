# P-20261104-01-patients-core: Núcleo de Pacientes

**Status:** Approved
**Created:** 2026-11-04
**Updated:** 2026-11-04
**Task list:** [todo.md](todo.md)

## Aprobación y límite de ejecución

El usuario aprobó el plan el 2026-11-04: «Lo veo bien, apruebo el plan. No comiences el desarrollo».

La aprobación fija el alcance y los IDs T01–T18/C01–C06. No autoriza iniciar el desarrollo. Todas las tareas permanecen Todo y todos los checkpoints siguen pendientes. Guardar estos documentos no completa T01 ni acredita pruebas de Pacientes.

## Overview

Implementar el componente Rust `patients` para registrar, consultar, buscar y actualizar pacientes, documentos y contactos responsables. Permitirá seguir editando y completar fichas sin conexión, conservar los cambios tras reiniciar y sincronizarlos con Nightingale API.

Decisiones confirmadas:
- Incluir contactos compartidos y menores sin documento propio.
- Usar el contrato actual `api-v1.1`.
- Entregar núcleo Rust, SQLite y comandos Tauri. La pantalla React queda fuera.
- Exigir pruebas finales con la API de desarrollo y datos ficticios.

Quedan fuera datos clínicos, fusión de duplicados, consentimiento de comunicación, Patient Hub, creación de citas, importación masiva y cambios en otros repositorios. No se editará la especificación como efecto del desarrollo.

## Source of Truth

- [Master Plan](../../docs/master-plan.md), §§9.4, 10–14, 19 y 23.
- [SRS](../../docs/srs.md): registro administrativo, datos mínimos y exclusión clínica.
- [Arquitectura técnica](../../docs/technical-architecture.md): operación local, límites y sincronización.
- [Decisión de tres proyectos](../../docs/decisions/transactional-sync-three-projects.md).
- [Contrato API fijo](../../docs/contracts/api-v1/nightingale-api.md), §§5.2, 5.5, 6–10, con sus esquemas y capacidades.
- [Contratos iniciales](../../docs/operations/core-next-tracks.md), apartado Pacientes y contactos.
- [Reglas de planificación](../README.md) y [CONTRIBUTING](../../CONTRIBUTING.md).
- Código actual: componentes services, identity y sync; DesktopProjector; migraciones 0001–0006 y registro SQLite.

Los planes de motor transversal y cuenta/catálogo están completados; este plan consume sus interfaces y no reabre su historial. El repositorio todavía no tiene un componente patients. La composición existente ya conecta AccountProjector y ServicesProjector con el ciclo normal y SSE.

Durante la exploración previa pasaron las cuatro pruebas existentes de `components::sync::contract_tests` y las cinco huellas del bundle. Esta evidencia no acredita funcionalidad de Pacientes ni la futura prueba API real.

## Architecture Decisions

### Propiedad, datos y límites

- patients posee dominio, casos de uso y adaptadores. Otros componentes consumen su API pública; no consultan sus tablas.
- Reservar `0007_patients`, sin modificar migraciones anteriores.
- Añadir cuatro vistas aceptadas: fichas, documentos, contactos y vínculos; una tabla de cambios locales con operaciones, dependencias y datos provisionales; índices por ámbito, estado, completitud y referencias.
- Todas las filas tendrán ámbito de cuenta, usuario y dispositivo. Remove/reset y una nueva captura conservarán las intenciones locales. No usar cascadas que las borren.
- Los datos creados o descargados ya locales estarán disponibles sin HTTP y tras reiniciar; un dispositivo nuevo necesita activación y descarga para obtener datos ajenos.
- Nightingale API posee aceptación compartida, contratos y PostgreSQL. El escritorio solo usa HTTPS y no recibe acceso SQL ni credenciales PostgreSQL.
- No hay excepción cloud del Hub en este plan. Hub, comunicaciones y sus servidores quedan fuera.
- El mismo responsable integra migraciones, módulos, comandos y permisos; no hay tareas paralelas autorizadas.

### Casos de uso e interfaces

Implementar estas once escrituras, sin ampliar el contrato remoto:

| Grupo | Acciones |
|---|---|
| Fichas | RegisterPatient, UpdatePatientProfile, SetPatientCompletion, SetPatientProfileStatus |
| Documentos | AddPatientIdentifier |
| Contactos | CreateContactPerson, UpdateContactPerson, SetContactPersonStatus |
| Vínculos | CreatePatientContactRelationship, UpdatePatientContactRelationship, SetPatientContactRelationshipStatus |

Añadir siete consultas locales: listar pacientes, consultar ficha, listar documentos, listar contactos de un paciente, listar contactos, consultar contacto y listar pacientes de un contacto.

Los 18 comandos Tauri usarán el prefijo `patients_`, nombres específicos y DTO camelCase. No admitirán SQL, credenciales, una cuenta arbitraria ni un envoltorio remoto libre. Rust obtendrá el ámbito de la sesión seleccionada.

Las consultas separarán:
- Estado aceptado y versión.
- Estado local provisional.
- Operaciones pendientes, conflictos, rechazos y dependencias bloqueadas.

Un puerto `PatientQuery` ofrecerá datos administrativos aceptados a futuros consumidores como Citas, sin modelos HTTP ni acceso al almacenamiento.

### Reglas del dominio

- Separar draft/complete de active/inactive; permitir fichas incompletas en draft conforme al contrato.
- Conservar documentos como texto, incluidos prefijos, ceros y puntuación.
- Mantener phoneCode/phone juntos: ambos presentes o ambos nulos.
- Distinguir ausencia y null al editar.
- No fusionar ni vincular personas por coincidencias de nombre, teléfono o documento.
- Añadir un documento propio posterior sin cambiar la identidad del paciente.
- No permitir que el cliente declare documentos o representación como verificados.
- Desactivar un paciente no cancela citas; un responsable válido no acredita tutela ni acceso al Hub.
- Comprobar completitud con nombre, base de identificación, documento o nacimiento y teléfono propio o responsable, según el contrato.
- La edad y los requisitos a la fecha de una cita pertenecen al futuro caso de uso de Citas.
- Una edición que deje fichas completas sin requisitos devuelve COMPLETENESS_CONFLICT con las referencias locales afectadas. El usuario debe pasarlas expresamente a draft; no hay cambio silencioso de completitud.
- Para fechas, reutilizar Chrono 0.4.45, presente en Cargo.lock, como dependencia directa con las funciones mínimas; no actualizar otras dependencias.

### Trabajo sin red, secuencias y permisos

Cada guardado confirmará el cambio local y su operación Outbox en una misma transacción SQLite. Propietario y asistente podrán operar según allowedActions y los últimos permisos válidos. La API revalida el permiso vigente al aceptar; la hora del equipo no prueba autorización. Una época nueva no reasigna intenciones previas.

Para editar varias veces sin red:
- Cada cambio tiene identidad y contenido propios.
- Rust enlaza cambios sucesivos mediante dependsOn.
- La base de una edición es una versión aceptada o la última operación local de esa entidad.
- Las versiones previstas para una secuencia son provisionales; solo API acredita versiones aceptadas.
- Evitar operaciones locales sin diferencia. Respetar el límite contractual de dependencias sin omitirlas para permitir el envío.
- Un rechazo o conflicto conserva la secuencia y señala sus pasos bloqueados. No ajustar versiones ni sobrescribir estado remoto de forma automática.
- Los cambios de contactos compartidos incorporan las dependencias necesarias de las operaciones explícitas de completitud.

### Descarga y conciliación

PatientsProjector traduce patient_profile, patient_identifier, contact_person y patient_contact; el vínculo usa la clave compuesta {patientId,contactPersonId}. Se incorpora a DesktopProjector para ciclo normal, SSE y recuperación por captura.

El proyector usa la transacción recibida, no hace HTTP ni modifica Outbox. Aplicar página/proyecciones/cursor o captura/baseCursor de forma atómica. Repeticiones y versiones antiguas no retroceden vistas.

Un recibo accepted no sustituye la descarga. La consulta retira un provisional confirmado cuando su vista aceptada alcanza la versión correspondiente, conservando operación y recibo.

Corregir un conflicto o rechazo conocido crea otra operación mediante el caso específico. Un resultado desconocido exige recuperar el resultado original antes de repetir la intención. La revisión conserva cambios bloqueados; no incluye una herramienta general de resolución automática de conflictos.

## Dependency Order

T01 → T02 → T03 → C01 → T04 → T05 → T06 → C02 → T07 → T08 → T09 → C03 → T10 → T11 → T12 → C04 → T13 → T14 → T15 → C05 → T16 → T17 → T18 → C06.

Un checkpoint pendiente detiene la fase siguiente hasta la revisión del usuario. Antes de cada cambio de código aplicar minimal-code tras trazar el flujo real y docs-first-framework-code cuando corresponda. El dominio local no depende de que el entorno remoto esté disponible.

## Task List

### Fase 1: Contratos y persistencia
- [ ] T01: Contratos del componente.
- [ ] T02: Dominio de fichas y formatos.
- [ ] T03: Persistencia local.

### C01: Revisión de contratos y persistencia
- [ ] Revisar interfaces, límites y migración antes de ampliar los casos de uso.
- [ ] Completar verificaciones de T01–T03.
- [ ] Revisión del usuario.

### Fase 2: Fichas
- [ ] T04: Registro y consulta de pacientes.
- [ ] T05: Ediciones sucesivas.
- [ ] T06: Documentos y completitud.

### C02: Revisión del recorrido de fichas
- [ ] Demostrar registro → edición → documento → completitud sin red y tras reinicio.
- [ ] Completar verificaciones de T04–T06.
- [ ] Revisión del usuario.

### Fase 3: Contactos
- [ ] T07: Contactos.
- [ ] T08: Vínculos.
- [ ] T09: Protección de fichas completas.

### C03: Revisión de contactos y responsables
- [ ] Demostrar menores y contactos compartidos, incluidos cambios que rompen requisitos.
- [ ] Completar verificaciones de T07–T09.
- [ ] Revisión del usuario.

### Fase 4: Sincronización
- [ ] T10: Proyecciones remotas.
- [ ] T11: Conciliación de cambios locales.
- [ ] T12: Integración del componente.

### C04: Revisión de sincronización
- [ ] Demostrar que resultados parciales o perdidos no duplican efectos ni borran cambios.
- [ ] Completar verificaciones de T10–T12.
- [ ] Revisión del usuario.

### Fase 5: Comandos nativos
- [ ] T13: Comandos de fichas.
- [ ] T14: Comandos de contactos.
- [ ] T15: Registro y permisos Tauri.

### C05: Revisión de la interfaz nativa
- [ ] Comprobar comandos, DTO y permisos sin añadir pantalla React.
- [ ] Completar verificaciones de T13–T15.
- [ ] Revisión del usuario.

### Fase 6: Verificación y cierre
- [ ] T16: Contrato del consumidor.
- [ ] T17: API de desarrollo real.
- [ ] T18: Documentación y calidad.

### C06: Aceptación de Pacientes
- [ ] Revisar resultados locales, prueba API real y límites expresos.
- [ ] Completar verificaciones de T16–T18.
- [ ] Revisión y aceptación del usuario.

## Test Plan

Cada tarea tendrá pruebas de comportamiento, comprobación de compilación y recorrido reproducible desde Rust o IPC. Los filtros nuevos se ligarán a pruebas reales en las tareas indicadas en todo.md.

Comandos previstos:
- `cargo test --workspace --locked components::patients`
- `cargo test --workspace --locked components::sync::contract_tests`
- `cargo test --workspace --locked patients_integration_tests`
- `pnpm quality`
- `pnpm build:desktop`
- `git diff --check`

Escenarios obligatorios:
- Guardar, editar varias veces y completar sin red; cerrar y reabrir.
- Fallar durante la escritura sin dejar solo la entidad o solo la operación.
- Adulto con documento propio y menor con responsable.
- Documento posterior del menor sin cambio de identidad.
- Contacto compartido y efectos sobre completitud.
- Repetición y respuesta perdida tras commit remoto.
- Aceptación parcial sin dar por aceptada toda la secuencia.
- Cambio de otro dispositivo con conservación de la intención en conflicto.
- Captura nueva y retiradas sin borrar trabajo pendiente.
- Aislamiento entre cuentas, permisos ausentes y revocación.
- API, Hub o SSE caídos sin impedir operaciones sobre datos locales.

El cierre requiere evidencia con API real de desarrollo. Si el entorno impide la prueba, registrar el bloqueo y mantener pendiente esa comprobación; las pruebas locales no la sustituyen. No se requiere cambiar los servidores ni probar Hub, comunicaciones, publicación o producción.

## Risks and Mitigations

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Diferencias entre estado provisional y aceptación de una secuencia | Alto | Dependencias explícitas, recibos individuales, conservación de bloqueados y prueba real. No ajustar versiones de forma automática. |
| Edición de contacto compartido que rompe requisitos | Alto | Evaluar impacto local, exigir draft explícito y ordenar dependencias; API revalida. |
| Respuesta perdida o resultado desconocido | Alto | Conservar ID/contenido y consultar el resultado original antes de otra intención. |
| Remove/reset o captura que borra trabajo | Alto | Separar vistas e intenciones y probar recuperación, aislamiento y rollback. |
| Copiar la regla de escritura exclusiva del propietario de Servicios | Alto | Comprobar allowedActions; probar propietario y asistente. |
| Entorno API/OIDC no disponible | Medio | Continuar pruebas locales independientes; no acreditar ni cerrar la verificación real. |
| Ampliar el alcance mediante funciones ausentes del contrato | Medio | Dejar fusión, consentimiento y cambios de API fuera de esta entrega. |

## Assumptions and Defaults

- Búsqueda local limitada a los datos disponibles de la cuenta; sin directorio global.
- Sin pantalla React ni diseño visual en este plan.
- Sin actualización del contrato ni de la especificación como efecto del trabajo.
- Solo datos ficticios. Cifrado y uso de datos reales requieren trabajo previo.
- Conservar trabajo ajeno y planes anteriores.
- No iniciar ninguna tarea hasta una nueva instrucción del usuario que autorice el desarrollo.

## Open Questions

Ninguna decisión de alcance pendiente. T01 concretará los contratos internos dentro del alcance aprobado; cualquier incompatibilidad que exija cambiarlo se presentará al usuario antes de implementar el punto afectado.
