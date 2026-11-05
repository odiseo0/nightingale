# C03: contactos, vínculos y protección de fichas completas

Fecha: 2026-11-05. C02 aprobado por el usuario. T07–T09 completos, en orden. **C03 pendiente de revisión; no aprobado. T10 no iniciado.**

## Cambios

- T07: creación, consulta, búsqueda, edición y actividad de contactos administrativos de la cuenta con las tres operaciones contractuales (`CreateContactPerson`, `UpdateContactPerson`, `SetContactPersonStatus`). `ContactData` exige nombre no vacío, pareja telefónica completa o nula y terna documental completa o nula; `ContactPatch` conserva ausencia/null, no permite retirar el nombre y aplica las reglas de par/terna sobre el valor resultante. Nombres y teléfonos iguales no fusionan identidades; un contacto puede servir a varios pacientes con una sola identidad.
- T08: vínculo paciente/contacto con clave compuesta `{patientId, contactPersonId}` y las tres operaciones contractuales. `RelationshipData` valida parentesco, función responsable y vigencia (offset RFC3339; `validUntil` posterior a `validFrom` cuando existe); `RelationshipPatch` permite que null solo retire `validUntil`. La creación exige ambos extremos en el ámbito e incorpora como dependencias las creaciones locales de paciente y contacto. Las consultas funcionan en ambos sentidos y devuelven nombres y teléfono actuales de lectura, que no se persisten en el estado ni en el payload del vínculo. El cliente no asigna ni verifica representación.
- Completitud offline: `completion_support` ahora acepta responsables provisionales cuyas operaciones aún pueden dependerse; un menor puede completar su ficha sin red con contacto y vínculo locales, y la operación de completitud incorpora las operaciones del vínculo (y del contacto si aplica) como dependencias.
- T09: las ediciones de contacto o vínculo que dejan una ficha completa sin sus requisitos devuelven `COMPLETENESS_CONFLICT` con las IDs locales afectadas y no guardan nada. Pasar expresamente las fichas a draft permite preparar el mismo cambio, que incorpora la operación explícita de completitud como dependencia. El límite contractual de ocho dependencias se respeta: preparar un cambio con nueve dependencias se rechaza sin perder pasos previos.
- Soporte común en `supports_at`/`responsible_impact`: evalúa, con el instante inyectado, qué fichas completas pierden el soporte del responsable; solo cuentan fichas que necesitan teléfono responsable (menor sin documento, o sin teléfono propio). Un adulto con documento propio y teléfono propio no se ve afectado por cambios del contacto.
- Se reutilizan permisos de Identidad, OperationContext, record_operation, reloj e IDs inyectados, dominio y tablas de C01. `edit_base` pasó a ser genérico sobre el estado de fila; `PreparedChange` acepta `patient_id` nulo y `contact_person_id`. No se añade migración ni dependencia en esta fase. Se conserva todo el trabajo local de C01–C02.

## Verificación

| Comprobación | Resultado |
|---|---|
| `cargo test --workspace --locked components::patients` al terminar T07 | 44 pruebas reales correctas |
| Mismo filtro al terminar T08 | 49 pruebas reales correctas |
| Mismo filtro al terminar T09 | 53 pruebas reales correctas |
| `cargo check --workspace --all-targets --locked` | Correcto al final de la fase |
| `cargo fmt --all -- --check` | Correcto |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Correcto |
| `cargo test --workspace --locked` final | 211 unitarias, 5 de integración y 1 de documentación correctas; 6 manuales ignoradas |
| `pnpm build:desktop` final | Correcto; genera target/debug/nightingale.exe |
| `git diff --check` | Correcto |
| SHA-256 del manifest | Las cinco huellas del bundle coinciden |

Las 16 pruebas nuevas (2 de dominio de contacto, 5 de casos de contacto, 1 de dominio de vínculo, 4 de casos de vínculo, 4 de protección de completitud) se suman a las 37 de C02. Las dos pruebas de migración pertenecen a `platform::sqlite` y también pasan en la suite completa. No se cuentan ejecuciones con cero pruebas como evidencia.

Recorridos ejecutados dentro de la suite y reproducibles por nombre:

```powershell
cargo test --workspace --locked components::patients::relationship_tests::minor_completes_offline_with_local_responsible_and_keeps_dependencies -- --exact
cargo test --workspace --locked components::patients::completeness_tests::removing_the_only_responsible_phone_conflicts_until_profiles_move_to_draft -- --exact
cargo test --workspace --locked components::patients::completeness_tests::shared_contact_conflict_lists_every_affected_complete_profile -- --exact
cargo test --workspace --locked components::patients::completeness_tests::dependency_limit_rejects_overloaded_shared_contact_edits_without_loss -- --exact
```

Se usan datos ficticios y archivos temporales. Las pruebas cubren permisos, cuenta/usuario/dispositivo, reapertura, rollback de intención+Outbox con disparador sintético, bases antiguas, NO_CHANGES, dependencias durables y la separación aceptado/provisional.

## Fuentes de las APIs usadas

Se aplicaron minimal-code y docs-first-framework-code, con versiones de Cargo.lock: rusqlite 0.40.2, Serde 1.0.229, serde_json 1.0.154 y Chrono 0.4.45. Esta fase no introduce APIs nuevas: reutiliza los patrones ya citados en la evidencia de C01–C02 ([rusqlite transactions](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html), [Serde derive](https://serde.rs/derive.html), [Chrono parse_from_rfc3339](https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.parse_from_rfc3339) y [from_timestamp_millis](https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.from_timestamp_millis)).

## Límites

No hay comandos Tauri o pantalla de Pacientes todavía. No se ejecutó API/OIDC real de Pacientes ni se acredita sincronización: proyector, conciliación y composición corresponden a T10–T12; la prueba remota corresponde a T17. Las búsquedas y la evaluación de impacto combinan las filas del ámbito antes de paginar/filtrar; no se ha medido su coste con cuentas grandes.

El impacto en completitud evalúa el estado visible (aceptado + última intención local) de las fichas del ámbito; no conoce el estado de otras cuentas ni afirma validez a fecha de una cita futura. El retiro de provisionales tras recibo/proyección y la presentación completa de dependencias bloqueadas queda para T11.

Contrato importado, especificaciones, React y migraciones previas intactos. No se abrió el ejecutable contra la base habitual del usuario ni se usaron datos reales. Sin commits ni push.

**Revisión pendiente:** validar el recorrido de menores/contactos compartidos y la protección de fichas completas antes de autorizar T10. La entrega de esta evidencia no aprueba C03.


## Corrigendum de la revisión C03 (2026-11-05)

La revisión de C03 (ejes Estándares y Spec) detectó tres defectos confirmados en T07–T09; se corrigieron con T19–T21 antes de la aprobación de C03. No se inicia T10.

- **T19** — `responsible_impact` evaluaba cada vínculo del contacto editado de forma aislada (falso `COMPLETENESS_CONFLICT` cuando la ficha conservaba otro responsable), no aplicaba el filtro `can_depend` que `completion_support` exige al soporte, releía los contactos por vínculo y convertía errores de almacenamiento en «sin teléfono» (`.ok()…unwrap_or`). Ahora el impacto se calcula por ficha con soporte residual, con las mismas reglas de dependencia que `completion_support`, lecturas únicas y propagación de errores; la cola de protección duplicada en los `prepare` de contacto y vínculo se extrajo a `documents::apply_impact`. Pruebas nuevas: `a_minor_with_two_responsibles_only_conflicts_when_the_last_phone_is_removed` y `a_rejected_responsible_link_neither_holds_nor_triggers_the_impact`.
- **T20** — la dependencia de la operación `SetPatientCompletion` solo se cableaba cuando era la última intención de la ficha; una edición posterior de la ficha la perdía. `intents::draft_move` localiza la operación explícita entre los cambios locales de la ficha. Prueba nueva: `a_profile_edit_after_the_draft_move_keeps_the_completion_dependency`.
- **T21** — `RelationshipPatch` coaccionaba `is_responsible_contact: Clear` a `false` y el payload emitía `isResponsibleContact: null`, pese a que el contrato fija que null solo retira `validUntil`. `apply` ahora rechaza `Clear` de responsabilidad y de parentesco con mensajes propios; el payload jamás emite ese null porque se construye solo tras aplicar el patch. Prueba de dominio ampliada (`relationship_data_validates_validity_and_patch_rules`).

Verificación tras las correcciones:

| Comprobación | Resultado |
|---|---|
| `cargo test --workspace --locked components::patients` | 56 pruebas reales correctas |
| `cargo test --workspace --locked` final | 214 unitarias, 5 de integración y 1 de documentación correctas; 6 manuales ignoradas |
| `cargo check --workspace --all-targets --locked` | Correcto |
| `cargo fmt --all -- --check` | Correcto |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Correcto |
| `pnpm build:desktop` | Correcto; genera `target/debug/nightingale.exe` |
| `git diff --check` | Correcto |

Sin API real ni afirmaciones sobre citas. C03 sigue pendiente de revisión y aprobación del usuario; T10 no iniciado.
