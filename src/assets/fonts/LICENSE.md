# Fuentes locales de Nightingale

## Aileron

- Diseñada por Sora Sagano (dot colon), https://www.dotcolon.net/font/aileron/
- Archivos: `aileron-latin-400-normal.woff2`, `aileron-latin-600-normal.woff2`.
- Licencia: uso libre, incluido uso comercial, según los términos publicados por
  dot colon ("License Public Domain / Free for Commercial Use"). No se incluye
  una licencia escrita en el paquete original de la fuente.
- Nota: la familia Aileron no distribuye un peso Medium. El peso 500 se sirve
  con el archivo SemiBold real (rango 500–600 en `@font-face`), sin síntesis de
  pesos.

## Anek Latin

- Diseñada por Ek Type, https://ektype.in/anek-latin.html
- Distribuida mediante el paquete npm `@fontsource/anek-latin` (archivos
  servidos localmente por Vite, sin CDN).
- Licencia: SIL Open Font License 1.1 (ver el paquete
  `node_modules/@fontsource/anek-latin`, que incluye el texto de la OFL y los
  archivos de copyright).

Ambas familias se sirven desde archivos locales empaquetados en la aplicación;
no hay solicitudes a CDN en tiempo de ejecución.
