# C06: aceptación de Pacientes

Fecha: 2026-11-05. C05 aprobado por el usuario («Continue» tras la entrega). T16 y T18 completos; **T17 preparada pero pendiente de su entorno real**. **C06 pendiente de revisión y aceptación del usuario.**

## Cambios

- T16: fixture `patients-operations.json` con las once acciones (payloads, claves, versiones esperadas y dependencias encadenadas) y prueba de consumidor que contrasta acciones, `payloadModels`, clave compuesta del vínculo, disponibilidad offline y rechazo de campos extra contra el bundle fijo. Las cinco huellas del bundle siguen intactas.
- T17: `components::patients::development_tests::manual_p17_responsible_contact_flow_with_two_devices` — prueba opt-in (`--ignored`) con adaptadores reales: login OIDC por navegador, dos dispositivos aislados, menor con responsable y completitud, ciclo de sincronización, descarga en el segundo dispositivo y conflicto concurrente que conserva intenciones. No ejecutada: falta Nightingale API/OIDC de desarrollo accesible desde esta máquina. Comando exacto: `$env:NIGHTINGALE_API_BASE_URL = "<url>"; cargo test --workspace --locked -- --ignored components::patients::development_tests`.
- T18: guía `docs/technical/components/patients.md`; `core-next-tracks.md` marca el track Pacientes como implementado; `development.md` refleja el estado y el comando opt-in.
- No se alteran contrato importado, especificaciones, React ni migraciones previas.

## Verificación

| Comprobación | Resultado |
|---|---|
| `cargo test --workspace --locked components::sync::contract_tests` (T16) | 5 pruebas reales correctas |
| `cargo test --workspace --locked components::patients` tras T17/T18 | 69 pruebas correctas, 1 manual ignorada (desarrollo real) |
| `cargo test --workspace --locked patients_integration_tests` | 1 prueba real correcta |
| `cargo test --workspace --locked` | 229 unitarias, 5 de integración y 1 de documentación correctas; 7 manuales ignoradas |
| `pnpm quality` | Correcto (build, fmt, clippy, suite completa) |
| `pnpm build:desktop` | Correcto; genera target/debug/nightingale.exe |
| `git diff --check` | Correcto |
| SHA-256 del manifest | Las cinco huellas del bundle coinciden |

## Qué queda verificado y qué no

Verificado localmente: núcleo Rust/SQLite de Pacientes, comandos Tauri y permisos, conciliación, proyecciones, integración de sincronización con servidor sintético y contrato del consumidor contra el bundle fijo.

**No verificado:** la prueba con Nightingale API/OIDC de desarrollo real (T17). Primer intento el 2026-11-05: la API respondió, pero el emisor OIDC usa certificado autofirmado y `NIGHTINGALE_DEV_CA_CERT` no estaba configurado, así que `discover` falló con `ProviderUnavailable` antes del login. Requiere exportar el CA de desarrollo (`NIGHTINGALE_DEV_CA_CERT`) y completar el login en navegador; el usuario retomará la ejecución. Un bloqueo no cuenta como éxito y las pruebas locales o sintéticas no la sustituyen. Sin pantalla React (fuera de alcance), sin datos clínicos, sin fusión de duplicados ni consentimiento de comunicación. `patient_local` conserva filas retiradas en lectura; el coste de acumulación se revisará si crece.

**Revisión pendiente:** revisar resultados locales, ejecutar T17 cuando haya entorno y aceptar o solicitar correcciones. La entrega de esta evidencia no aprueba C06.
