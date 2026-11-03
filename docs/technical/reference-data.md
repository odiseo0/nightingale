# Datos de referencia compartidos

Las listas pertenecen a
[`shared_kernel/reference_data`](../../src-tauri/src/shared_kernel/reference_data).
Los componentes consultan `is_country_code(&str)` e `is_time_zone(&str)` para validar
pertenencia. Ambas funciones usan datos incluidos al compilar: no leen archivos del
dispositivo ni necesitan red. Las reglas propias de cada caso permanecen en su dominio.

- [Códigos de país](../../src-tauri/src/shared_kernel/reference_data/countries.txt).
- [Nombres de zona horaria](../../src-tauri/src/shared_kernel/reference_data/time_zones.txt).

Fuente: https://data.iana.org/time-zones/tzdata-latest.tar.gz

Versión tzdb: 2026f. SHA-256 del archivo: 01b4a91cec5167ed48c11e8ab78f2048cead78ce94b0cafcad0ae8c5a3044d3d.

Los 598 nombres de time_zones.txt son Zone y Link de los archivos regionales, etcetera, backward y factory. countries.txt contiene los 249 códigos ISO alfa-2 de iso3166.tab. Solo se usan para pertenencia; no se calculan fechas ni offsets. Datos de IANA de dominio público. Actualizar ambas listas juntas desde una versión revisada si IANA añade zonas o cambia códigos. La API vuelve a validar antes de aceptar.
