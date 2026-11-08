import { useState, type FormEvent } from "react";
import {
  checkRemoteAvailability, getLocalAvailability,
  type AvailabilityEvaluation, type RemoteAvailability,
} from "../../shared/api/scheduling";

type LocalState =
  | { status: "idle" }
  | { status: "loading"; source: "local" | "remote" }
  | { status: "local"; evaluation: AvailabilityEvaluation }
  | { status: "remote"; result: RemoteAvailability }
  | { status: "error"; code: string; source: "local" | "remote" };

const coverageLabel: Record<AvailabilityEvaluation["coverage"], string> = {
  within_synced_window: "Dentro de la ventana descargada (aun provisional).",
  no_synced_window: "Sin ventana descargada: resultado solo local para trabajo inicial sin conexión.",
  insufficient_coverage: "Cobertura insuficiente: no se puede distinguir de «sin huecos»; considera la comprobación con API.",
};

const asUtc = (iso: string) => iso;
const inZone = (iso: string, timeZone: string) =>
  new Intl.DateTimeFormat("es-VE", { dateStyle: "short", timeStyle: "short", timeZone }).format(new Date(iso));

export default function AvailabilityPanel() {
  const [state, setState] = useState<LocalState>({ status: "idle" });
  const [localZone, setLocalZone] = useState(true);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const d = new FormData(event.currentTarget);
    const request = {
      professionalId: String(d.get("professional") ?? ""),
      serviceId: String(d.get("service") ?? ""),
      from: new Date(String(d.get("from"))).toISOString(),
      until: new Date(String(d.get("until"))).toISOString(),
      ...(String(d.get("duration") || "") === "" ? {} : { durationMinutes: Number(d.get("duration")) }),
    };
    setState({ status: "loading", source: "local" });
    try {
      setState({ status: "local", evaluation: await getLocalAvailability(request) });
    } catch (error: unknown) {
      const code = typeof error === "object" && error !== null && "code" in error ? String((error as { code: string }).code) : "TEMPORARY_FAILURE";
      setState({ status: "error", code, source: "local" });
    }
  }

  async function checkRemote(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const d = new FormData(event.currentTarget);
    const request = {
      professionalId: String(d.get("professional") ?? ""),
      serviceId: String(d.get("service") ?? ""),
      from: new Date(String(d.get("from"))).toISOString(),
      until: new Date(String(d.get("until"))).toISOString(),
      ...(String(d.get("duration") || "") === "" ? {} : { durationMinutes: Number(d.get("duration")) }),
    };
    setState({ status: "loading", source: "remote" });
    try {
      // The remote result is the current API view: it never reserves and is
      // not stored; on error the local evaluation above stays valid.
      setState({ status: "remote", result: await checkRemoteAvailability(request) });
    } catch (error: unknown) {
      const code = typeof error === "object" && error !== null && "code" in error ? String((error as { code: string }).code) : "TEMPORARY_FAILURE";
      setState({ status: "error", code, source: "remote" });
    }
  }

  return <>
    <form onSubmit={submit} className="grid gap-3 sm:grid-cols-3">
      <label className="grid gap-1">Profesional (UUID)<input name="professional" required /></label>
      <label className="grid gap-1">Servicio (UUID)<input name="service" required /></label>
      <label className="grid gap-1">Duración (minutos, opcional)<input name="duration" type="number" min="1" max="1440" /></label>
      <label className="grid gap-1">Desde<input name="from" type="datetime-local" required /></label>
      <label className="grid gap-1">Hasta<input name="until" type="datetime-local" required /></label>
      <button type="submit">Consultar disponibilidad local</button>
    </form>

    {state.status === "loading" && <p role="status">{state.source === "local" ? "Calculando disponibilidad local…" : "Consultando la API…"}</p>}
    {state.status === "error" && (
      <p role="alert">
        {state.source === "remote"
          ? `La consulta remota falló (${state.code}); la vista local, si existe, se conserva.`
          : `No se pudo evaluar la disponibilidad (${state.code}).`}
      </p>
    )}

    {state.status === "local" && (
      <div className="space-y-2">
        <p role="status">
          <strong>{coverageLabel[state.evaluation.coverage]}</strong>{" "}
          Zona {state.evaluation.timeZone}; duración de referencia {state.evaluation.referenceDurationMinutes} min, solicitada {state.evaluation.requestedDurationMinutes} min.
          Resultado siempre provisional: puede haber cambios no recibidos y ocupación fuera de la ventana.
        </p>
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" checked={localZone} onChange={(e) => setLocalZone(e.target.checked)} /> Mostrar en la zona de la cuenta
        </label>
        {state.evaluation.intervals.length === 0 ? (
          <p>Sin intervalos que contengan la duración solicitada{coverageLabel[state.evaluation.coverage].startsWith("Cobertura") ? " (con cobertura insuficiente, esto no prueba ausencia de huecos)" : ""}.</p>
        ) : (
          <ul className="space-y-1 text-sm">
            {state.evaluation.intervals.map((interval) => (
              <li key={interval.startsAt}>
                {localZone
                  ? `${inZone(interval.startsAt, state.evaluation.timeZone)} – ${inZone(interval.endsAt, state.evaluation.timeZone)}`
                  : `${asUtc(interval.startsAt)} – ${asUtc(interval.endsAt)}`}
                {!localZone && " (UTC)"}
              </li>
            ))}
          </ul>
        )}
      </div>
    )}

    <form onSubmit={checkRemote} className="grid gap-3 border-t border-gray-200 pt-4 sm:grid-cols-3">
      <label className="grid gap-1">Profesional (UUID)<input name="professional" required /></label>
      <label className="grid gap-1">Servicio (UUID)<input name="service" required /></label>
      <label className="grid gap-1">Duración (minutos, opcional)<input name="duration" type="number" min="1" max="1440" /></label>
      <label className="grid gap-1">Desde<input name="from" type="datetime-local" required /></label>
      <label className="grid gap-1">Hasta<input name="until" type="datetime-local" required /></label>
      <button type="submit">Comprobar con la API</button>
    </form>

    {state.status === "remote" && (
      <p role="status">
        Vista vigente de la API (no reserva ni se guarda): {state.result.intervals.length} intervalo(s) de{" "}
        {state.result.referenceDurationMinutes} min de referencia.
      </p>
    )}
  </>;
}
