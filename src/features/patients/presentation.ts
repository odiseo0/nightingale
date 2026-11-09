/** UUID v7 mínimo para los ids de identificador exigidos por el contrato
 * de registro (el servidor valida el formato, no la procedencia). */
export function uuidV7(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  const millis = Date.now();
  bytes[0] = (millis / 2 ** 40) & 0xff;
  bytes[1] = (millis / 2 ** 32) & 0xff;
  bytes[2] = (millis / 2 ** 24) & 0xff;
  bytes[3] = (millis / 2 ** 16) & 0xff;
  bytes[4] = (millis / 2 ** 8) & 0xff;
  bytes[5] = millis & 0xff;
  bytes[6] = (bytes[6] & 0x0f) | 0x70;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = [...bytes].map((b) => b.toString(16).padStart(2, "0"));
  return `${hex.slice(0, 4).join("")}-${hex.slice(4, 6).join("")}-${hex.slice(6, 8).join("")}-${hex.slice(8, 10).join("")}-${hex.slice(10).join("")}`;
}

export const relationshipKindLabel: Record<string, string> = {
  mother: "Madre",
  father: "Padre",
  guardian: "Tutor/a",
  other: "Otro",
};

export const documentTypeLabel: Record<string, string> = {
  national_id: "Documento nacional",
  passport: "Pasaporte",
};
