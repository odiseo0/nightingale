import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  createBlock, createException, createRule, listBlocks, listExceptions, listRules,
  setBlockStatus, setExceptionStatus, setRuleStatus, updateBlock, updateException, updateRule,
  type BlockState, type EntityRow, type ExceptionState, type LocalMutationResult,
  type RuleState, type ScheduleStatus, visibleOf,
} from "../../shared/api/scheduling";

const weekday = (iso: number) => ["", "Lunes", "Martes", "Miércoles", "Jueves", "Viernes", "Sábado", "Domingo"][iso] ?? iso;
const minute = (value: number) => `${String(Math.floor(value / 60)).padStart(2, "0")}:${String(value % 60).padStart(2, "0")}`;
const reason: Record<string, string> = { vacation: "Vacaciones", administrative: "Administrativo", unavailable: "No disponible" };
const syncLabel: Record<string, string> = {
  pending: "Pendiente", in_flight: "Enviando", unknown: "Resultado desconocido", retryable: "Esperando reintento",
  accepted: "Aceptada; falta confirmar la vista", conflict: "Conflicto", rejected: "Rechazada", review_required: "Requiere revisión",
};

type AgendaData = {
  rules: EntityRow<RuleState>[];
  exceptions: EntityRow<ExceptionState>[];
  blocks: EntityRow<BlockState>[];
};

type EditorKind = "rule" | "exception" | "block";

export default function AgendaEditor() {
  const [data, setData] = useState<AgendaData>({ rules: [], exceptions: [], blocks: [] });
  const [kind, setKind] = useState<EditorKind>("rule");
  const [editing, setEditing] = useState<{ id: string; base: { operationId?: string; acceptedVersion?: number } } | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [showInactive, setShowInactive] = useState(false);
  const generation = useRef(0);

  async function refresh() {
    const request = ++generation.current;
    try {
      const [rules, exceptions, blocks] = await Promise.all([listRules(), listExceptions(), listBlocks()]);
      if (request === generation.current) setData({ rules, exceptions, blocks });
    } catch { /* session not ready yet */ }
  }

  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => void refresh(), 2000);
    return () => { ++generation.current; window.clearInterval(timer); };
  }, []);

  async function act(work: () => Promise<LocalMutationResult>, success = "Cambio guardado en este dispositivo; pendiente de aceptación.") {
    setBusy(true); setMessage("");
    try {
      await work(); setMessage(success); setEditing(null); void refresh();
    } catch (error: unknown) {
      const code = typeof error === "object" && error !== null && "code" in error ? String((error as { code: string }).code) : "TEMPORARY_FAILURE";
      setMessage(code === "SLOT_UNAVAILABLE"
        ? "El cambio deja una cita conocida fuera del horario o dentro de un bloqueo; no se guardó."
        : `No se pudo completar la acción (${code}). Revisa el acceso, las versiones y los cambios pendientes.`);
    } finally { setBusy(false); }
  }

  function baseOf<T>(row: EntityRow<T>) {
    return row.provisional ? { operationId: row.provisional.operationId } : { acceptedVersion: row.accepted!.version };
  }

  const visible = { rule: visibleOf<RuleState>, exception: visibleOf<ExceptionState>, block: visibleOf<BlockState> };

  function toggleRule(row: EntityRow<RuleState>, next: ScheduleStatus) {
    const state = visible.rule(row);
    if (!state) return;
    void act(() => setRuleStatus({ ruleId: state.ruleId, base: baseOf(row), status: next }));
  }

  function toggleException(row: EntityRow<ExceptionState>, next: ScheduleStatus) {
    const state = visible.exception(row);
    if (!state) return;
    void act(() => setExceptionStatus({ exceptionId: state.exceptionId, base: baseOf(row), status: next }));
  }

  function toggleBlock(row: EntityRow<BlockState>, next: ScheduleStatus) {
    const state = visible.block(row);
    if (!state) return;
    void act(() => setBlockStatus({ blockId: state.blockId, base: baseOf(row), status: next }));
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const d = new FormData(form);
    const professional = String(d.get("professional") ?? "");
    if (kind === "rule") {
      const patch = {
        isoWeekday: Number(d.get("isoWeekday")),
        localStartMinute: Number(d.get("start")),
        localEndMinute: Number(d.get("end")),
        validFrom: String(d.get("validFrom")),
        ...(String(d.get("validUntil") || "") === "" ? {} : { validUntil: String(d.get("validUntil")) }),
      };
      void act(() => editing
        ? updateRule({ ruleId: editing.id, base: editing.base, patch })
        : createRule({ professionalId: professional, ...patch }));
    } else if (kind === "exception") {
      const closed = d.get("kind") === "closed";
      const patch = closed
        ? { kind: "closed" as const }
        : { kind: "replacement" as const, localStartMinute: Number(d.get("start")), localEndMinute: Number(d.get("end")) };
      void act(() => editing
        ? updateException({ exceptionId: editing.id, base: editing.base, patch })
        : createException({ professionalId: professional, localDate: String(d.get("localDate")), ...patch }));
    } else {
      const patch = {
        startsAt: String(d.get("startsAt")),
        endsAt: String(d.get("endsAt")),
        reasonCode: String(d.get("reasonCode")) as "vacation" | "administrative" | "unavailable",
      };
      void act(() => editing
        ? updateBlock({ blockId: editing.id, base: editing.base, patch })
        : createBlock({
            professionalId: professional,
            ...(d.get("global") === "on" ? { accountId: null } : {}),
            ...patch,
          }));
    }
    form.reset();
  }

  return <>
    <form onSubmit={submit} className="grid gap-3 sm:grid-cols-3">
      <fieldset disabled={busy} className="contents">
        <label className="grid gap-1">Elemento
          <select value={kind} onChange={(e) => { setKind(e.target.value as EditorKind); setEditing(null); }}>
            <option value="rule">Regla semanal</option>
            <option value="exception">Excepción por fecha</option>
            <option value="block">Bloqueo</option>
          </select>
        </label>
        <label className="grid gap-1">Profesional (UUID)
          <input name="professional" required placeholder="UUID del profesional" disabled={!!editing} />
        </label>
        {kind === "rule" && <>
          <label className="grid gap-1">Día ISO (1=lunes … 7=domingo)<input name="isoWeekday" type="number" min="1" max="7" required /></label>
          <label className="grid gap-1">Inicio (minuto 0–1439)<input name="start" type="number" min="0" max="1439" required /></label>
          <label className="grid gap-1">Fin (minuto 1–1440)<input name="end" type="number" min="1" max="1440" required /></label>
          <label className="grid gap-1">Vigente desde<input name="validFrom" type="date" required /></label>
          <label className="grid gap-1">Vigente hasta (opcional)<input name="validUntil" type="date" /></label>
        </>}
        {kind === "exception" && <>
          <label className="grid gap-1">Fecha<input name="localDate" type="date" required /></label>
          <label className="grid gap-1">Tipo
            <select name="kind"><option value="closed">Cerrado (día vacío)</option><option value="replacement">Reemplazo de horario</option></select>
          </label>
          <label className="grid gap-1">Inicio (minutos)<input name="start" type="number" min="0" max="1439" /></label>
          <label className="grid gap-1">Fin (minutos)<input name="end" type="number" min="1" max="1440" /></label>
        </>}
        {kind === "block" && <>
          <label className="grid gap-1">Inicio (RFC 3339)<input name="startsAt" type="datetime-local" required /></label>
          <label className="grid gap-1">Fin (RFC 3339)<input name="endsAt" type="datetime-local" required /></label>
          <label className="grid gap-1">Motivo
            <select name="reasonCode"><option value="vacation">Vacaciones</option><option value="administrative">Administrativo</option><option value="unavailable">No disponible</option></select>
          </label>
          <label className="flex items-center gap-2">Global (todas las cuentas)<input name="global" type="checkbox" disabled={!!editing} /></label>
        </>}
        <button type="submit">{editing ? "Guardar edición" : "Crear local"}</button>
        {editing && <button type="button" onClick={() => setEditing(null)}>Cancelar edición</button>}
      </fieldset>
    </form>
    <p role="status">{message}</p>
    <label className="flex items-center gap-2 text-sm">
      <input type="checkbox" checked={showInactive} onChange={(e) => setShowInactive(e.target.checked)} /> Mostrar inactivos
    </label>

    <h3>Reglas semanales</h3>
    <ul className="space-y-2 text-sm">
      {data.rules.map((row) => {
        const state = visible.rule(row);
        return state && (showInactive || state.status !== "inactive") && (
        <li key={state.ruleId} className="space-y-1">
          <strong>{weekday(state.isoWeekday)}</strong> {minute(state.localStartMinute)}–{minute(state.localEndMinute)} ·
          vigencia {state.validFrom}{state.validUntil ? `–${state.validUntil}` : " en adelante"} ·
          {state.status === "active" ? " activa" : " inactiva"}
          {row.provisional && <em> (provisional)</em>}
          <Changes ops={row.operations} />
          <span className="flex gap-2">
            <button disabled={busy} onClick={() => toggleRule(row, state.status === "active" ? "inactive" : "active")}>
              {state.status === "active" ? "Desactivar" : "Activar"}
            </button>
          </span>
        </li>
        );
      })}
    </ul>

    <h3>Excepciones</h3>
    <ul className="space-y-2 text-sm">
      {data.exceptions.map((row) => {
        const state = visible.exception(row);
        return state && (showInactive || state.status !== "inactive") && (
        <li key={state.exceptionId}>
          {state.localDate} · {state.kind === "closed" ? "cerrado" : `reemplazo ${minute(state.localStartMinute ?? 0)}–${minute(state.localEndMinute ?? 0)}`}
          {state.status === "inactive" && " · inactiva"}{row.provisional && <em> (provisional)</em>}
          <Changes ops={row.operations} />
          <button disabled={busy} onClick={() => toggleException(row, state.status === "active" ? "inactive" : "active")}>
            {state.status === "active" ? "Desactivar" : "Activar"}
          </button>
        </li>
        );
      })}
    </ul>

    <h3>Bloqueos</h3>
    <ul className="space-y-2 text-sm">
      {data.blocks.map((row) => {
        const state = visible.block(row);
        return state && (showInactive || state.status !== "inactive") && (
        <li key={state.blockId}>
          {state.startsAt} – {state.endsAt} · {state.blockAccountId ? "de cuenta" : "global"} ·
          {state.reasonCode ? ` ${reason[state.reasonCode]}` : " motivo no disponible para este acceso"}
          {state.status === "inactive" && " · inactiva"}{row.provisional && <em> (provisional)</em>}
          <Changes ops={row.operations} />
          <button disabled={busy} onClick={() => toggleBlock(row, state.status === "active" ? "inactive" : "active")}>
            {state.status === "active" ? "Desactivar" : "Activar"}
          </button>
        </li>
        );
      })}
    </ul>
  </>;
}

function Changes({ ops }: { ops: { operationId: string; status: string; error: string | null }[] }) {
  if (!ops.length) return null;
  return <ul aria-label="Cambios locales" className="space-y-1">
    {ops.map((op) => (
      <li key={op.operationId} className="break-all">
        {syncLabel[op.status] ?? op.status}{op.error ? ` · ${op.error}` : ""} · <code>{op.operationId}</code>
        {["conflict", "rejected", "review_required"].includes(op.status) && <> · <a href="#core-heading">Ver revisión de sincronización</a></>}
      </li>
    ))}
  </ul>;
}
