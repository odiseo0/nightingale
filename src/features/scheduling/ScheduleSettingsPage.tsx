import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";
import { CircleNotch, Lock, Plus, Power } from "@phosphor-icons/react";
import {
  createBlock, createException, createRule, listBlocks, listExceptions, listRules,
  setBlockStatus, setExceptionStatus, setRuleStatus,
  type BlockState, type EntityRow, type ExceptionState, type RuleState,
} from "../../shared/api/scheduling";
import { getBookingContext } from "../../shared/api/appointments";
import { getAccountView } from "../../shared/api/account";
import { mutationErrorMessage } from "../appointments/presentation";

const weekdayLabel = ["Lunes", "Martes", "Miércoles", "Jueves", "Viernes", "Sábado", "Domingo"];

const blockReasonLabel: Record<string, string> = {
  vacation: "Vacaciones",
  administrative: "Administrativo",
  unavailable: "No disponible",
};

const minutesToTime = (minutes: number) =>
  `${String(Math.floor(minutes / 60)).padStart(2, "0")}:${String(minutes % 60).padStart(2, "0")}`;
const timeToMinutes = (time: string) => {
  const [h, m] = time.split(":").map(Number);
  return h * 60 + m;
};

function baseOf<T>(row: EntityRow<T>): { acceptedVersion?: number; operationId?: string } {
  return row.provisional
    ? { operationId: row.provisional.operationId }
    : { acceptedVersion: row.accepted!.version };
}

/** S12 · Horarios: reglas semanales, excepciones y bloqueos con controles
 * nativos; la conversión a minutos/instantes ocurre solo en la frontera DTO. */
export default function ScheduleSettingsPage() {
  const [rules, setRules] = useState<EntityRow<RuleState>[] | null>(null);
  const [exceptions, setExceptions] = useState<EntityRow<ExceptionState>[] | null>(null);
  const [blocks, setBlocks] = useState<EntityRow<BlockState>[] | null>(null);
  const [professionalId, setProfessionalId] = useState("");
  const [accountId, setAccountId] = useState<string | null>(null);
  const [phase, setPhase] = useState<"loading" | "ready" | "error">("loading");
  const [editor, setEditor] = useState<"none" | "rule" | "exception" | "block">("none");
  const [ruleDraft, setRuleDraft] = useState({ weekday: "2", start: "09:00", end: "12:00", validFrom: "", validUntil: "" });
  const [exceptionDraft, setExceptionDraft] = useState({ date: "", kind: "closed", start: "09:00", end: "12:00" });
  const [blockDraft, setBlockDraft] = useState({ start: "", end: "", reason: "vacation", global: false });
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const savingRef = useRef(false);
  const generation = useRef(0);

  const load = useCallback(async () => {
    const request = ++generation.current;
    try {
      const [ruleRows, exceptionRows, blockRows, context, screen] = await Promise.all([
        listRules(), listExceptions(), listBlocks(), getBookingContext(), getAccountView(),
      ]);
      if (request !== generation.current) return;
      setRules(ruleRows);
      setExceptions(exceptionRows);
      setBlocks(blockRows);
      setProfessionalId(context.professionals[0]?.accepted?.professionalId ?? "");
      setAccountId(screen.account?.accountId ?? null);
      setPhase("ready");
    } catch {
      if (request === generation.current) setPhase("error");
    }
  }, []);

  useEffect(() => {
    void load();
    const timer = window.setInterval(() => void load(), 10_000);
    return () => { generation.current += 1; window.clearInterval(timer); };
  }, [load]);

  async function run(work: () => Promise<unknown>, success = "Cambios guardados.") {
    if (savingRef.current) return;
    savingRef.current = true;
    setSaving(true);
    setFormError(null);
    try {
      await work();
      setNotice(success);
      setEditor("none");
      await load();
    } catch (error: unknown) {
      setFormError(mutationErrorMessage(error));
    } finally {
      savingRef.current = false;
      setSaving(false);
    }
  }

  function submitRule(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const start = timeToMinutes(ruleDraft.start);
    const end = timeToMinutes(ruleDraft.end);
    if (end <= start) { setFormError("La hora de fin debe ser posterior al inicio."); return; }
    void run(() => createRule({
      professionalId,
      isoWeekday: Number(ruleDraft.weekday),
      localStartMinute: start,
      localEndMinute: end,
      validFrom: ruleDraft.validFrom || new Date().toISOString().slice(0, 10),
      ...(ruleDraft.validUntil ? { validUntil: ruleDraft.validUntil } : {}),
    }), "Regla guardada.");
  }

  function submitException(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!exceptionDraft.date) { setFormError("Indica la fecha de la excepción."); return; }
    void run(() => createException({
      professionalId,
      localDate: exceptionDraft.date,
      kind: exceptionDraft.kind as "closed" | "replacement",
      ...(exceptionDraft.kind === "replacement"
        ? { localStartMinute: timeToMinutes(exceptionDraft.start), localEndMinute: timeToMinutes(exceptionDraft.end) }
        : {}),
    }), "Excepción guardada.");
  }

  function submitBlock(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!blockDraft.start || !blockDraft.end) { setFormError("Indica inicio y fin del bloqueo."); return; }
    void run(() => createBlock({
      professionalId,
      /* El contrato usa accountId ausente/null = bloqueo global; el bloqueo de
         la cuenta exige el identificador de la cuenta activa. */
      ...(blockDraft.global || !accountId
        ? { accountId: null }
        : { accountId }),
      startsAt: new Date(blockDraft.start).toISOString(),
      endsAt: new Date(blockDraft.end).toISOString(),
      reasonCode: blockDraft.reason as "vacation" | "administrative" | "unavailable",
    }), "Bloqueo guardado.");
  }

  if (phase === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Cargando los horarios…</p>
      </div>
    );
  }

  if (phase === "error") {
    return (
      <div className="state-block" role="alert">
        <p>No se pudo leer la agenda local.</p>
        <button type="button" className="btn" onClick={() => void load()}>Reintentar</button>
      </div>
    );
  }

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">Horarios y bloqueos</h1>
          <p className="muted">Reglas semanales, excepciones por fecha y bloqueos de la cuenta.</p>
        </div>
      </div>

      {notice && <div className="notice" role="status"><p>{notice}</p></div>}
      {formError && <div className="notice" role="alert"><p>{formError}</p></div>}

      <p className="notice">
        <Lock size={16} aria-hidden="true" style={{ verticalAlign: -3 }} />{" "}
        Los cambios de horario no mueven ni cancelan citas existentes. Si una cita quedara fuera del
        horario o dentro de un bloqueo, el cambio se rechaza y el formulario se conserva para corregirlo.
      </p>

      {/* Reglas */}
      <section aria-labelledby="reglas-titulo">
        <div className="page-header">
          <h2 id="reglas-titulo" style={{ fontSize: 18 }}>Reglas semanales</h2>
          {editor !== "rule" && (
            <button type="button" className="btn btn--primary"
              onClick={() => { setEditor("rule"); setRuleDraft({ weekday: "2", start: "09:00", end: "12:00", validFrom: "", validUntil: "" }); }}>
              <Plus size={16} aria-hidden="true" />
              Nueva regla
            </button>
          )}
        </div>
        {editor === "rule" && (
          <form className="card" style={{ padding: 16, display: "grid", gap: 12, marginBottom: 12 }} onSubmit={(e) => void submitRule(e)}>
            <div className="toolbar">
              <label className="field-label">
                Día de la semana
                <select className="select" value={ruleDraft.weekday}
                  onChange={(e) => setRuleDraft({ ...ruleDraft, weekday: e.target.value })}>
                  {weekdayLabel.map((label, index) => (
                    <option key={label} value={index + 1}>{label}</option>
                  ))}
                </select>
              </label>
              <label className="field-label">
                Inicio
                <input type="time" className="input-date" value={ruleDraft.start}
                  onChange={(e) => setRuleDraft({ ...ruleDraft, start: e.target.value })} />
              </label>
              <label className="field-label">
                Fin
                <input type="time" className="input-date" value={ruleDraft.end}
                  onChange={(e) => setRuleDraft({ ...ruleDraft, end: e.target.value })} />
              </label>
            </div>
            <div className="toolbar">
              <label className="field-label">
                Válida desde
                <input type="date" className="input-date" value={ruleDraft.validFrom}
                  onChange={(e) => setRuleDraft({ ...ruleDraft, validFrom: e.target.value })} />
              </label>
              <label className="field-label">
                Válida hasta (opcional)
                <input type="date" className="input-date" value={ruleDraft.validUntil}
                  onChange={(e) => setRuleDraft({ ...ruleDraft, validUntil: e.target.value })} />
              </label>
            </div>
            <div className="toolbar">
              <button type="submit" className="btn btn--primary" disabled={saving}>Guardar regla</button>
              <button type="button" className="btn" onClick={() => setEditor("none")}>Cancelar</button>
            </div>
          </form>
        )}
        {!rules || rules.length === 0 ? <p className="muted">Sin reglas semanales.</p> : (
          <div className="table-scroll">
            <table className="data-table">
              <thead>
                <tr><th scope="col">Día</th><th scope="col">Horario</th><th scope="col">Vigencia</th><th scope="col">Estado</th><th scope="col">Acciones</th></tr>
              </thead>
              <tbody>
                {rules.map((row) => {
                  const rule = row.accepted?.state ?? row.provisional?.state ?? null;
                  if (!rule) return null;
                  const busy = row.provisional !== null;
                  return (
                    <tr key={rule.ruleId}>
                      <td>{weekdayLabel[rule.isoWeekday - 1] ?? rule.isoWeekday}</td>
                      <td className="num">{minutesToTime(rule.localStartMinute)}–{minutesToTime(rule.localEndMinute)}</td>
                      <td className="num">{rule.validFrom}{rule.validUntil ? ` – ${rule.validUntil}` : " en adelante"}</td>
                      <td><span className="badge">{rule.status === "active" ? "Activa" : "Inactiva"}</span></td>
                      <td>
                        <div className="toolbar" style={{ gap: 6 }}>
                          <button type="button" className="btn btn--subtle" disabled={busy || saving}
                            onClick={() => void run(() => setRuleStatus({
                              ruleId: rule.ruleId, base: baseOf(row),
                              status: rule.status === "active" ? "inactive" : "active",
                            }), `Regla ${rule.status === "active" ? "desactivada" : "activada"}.`)}>
                            <Power size={14} aria-hidden="true" />
                            {rule.status === "active" ? "Desactivar" : "Activar"}
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {/* Excepciones */}
      <section aria-labelledby="excepciones-titulo" style={{ marginTop: 24 }}>
        <div className="page-header">
          <h2 id="excepciones-titulo" style={{ fontSize: 18 }}>Excepciones por fecha</h2>
          {editor !== "exception" && (
            <button type="button" className="btn"
              onClick={() => { setEditor("exception"); setExceptionDraft({ date: "", kind: "closed", start: "09:00", end: "12:00" }); }}>
              <Plus size={16} aria-hidden="true" />
              Nueva excepción
            </button>
          )}
        </div>
        {editor === "exception" && (
          <form className="card" style={{ padding: 16, display: "grid", gap: 12, marginBottom: 12 }} onSubmit={(e) => void submitException(e)}>
            <p className="muted" style={{ fontSize: 13 }}>Una excepción cambia la atención de un día concreto: cierre total o horario de reemplazo.</p>
            <div className="toolbar">
              <label className="field-label">
                Fecha
                <input type="date" className="input-date" value={exceptionDraft.date}
                  onChange={(e) => setExceptionDraft({ ...exceptionDraft, date: e.target.value })} />
              </label>
              <label className="field-label">
                Tipo
                <select className="select" value={exceptionDraft.kind}
                  onChange={(e) => setExceptionDraft({ ...exceptionDraft, kind: e.target.value })}>
                  <option value="closed">Cierre (día completo)</option>
                  <option value="replacement">Horario de reemplazo</option>
                </select>
              </label>
              {exceptionDraft.kind === "replacement" && (
                <>
                  <label className="field-label">
                    Inicio
                    <input type="time" className="input-date" value={exceptionDraft.start}
                      onChange={(e) => setExceptionDraft({ ...exceptionDraft, start: e.target.value })} />
                  </label>
                  <label className="field-label">
                    Fin
                    <input type="time" className="input-date" value={exceptionDraft.end}
                      onChange={(e) => setExceptionDraft({ ...exceptionDraft, end: e.target.value })} />
                  </label>
                </>
              )}
            </div>
            <div className="toolbar">
              <button type="submit" className="btn btn--primary" disabled={saving}>Guardar excepción</button>
              <button type="button" className="btn" onClick={() => setEditor("none")}>Cancelar</button>
            </div>
          </form>
        )}
        {!exceptions || exceptions.length === 0 ? <p className="muted">Sin excepciones registradas.</p> : (
          <div className="table-scroll">
            <table className="data-table">
              <thead>
                <tr><th scope="col">Fecha</th><th scope="col">Tipo</th><th scope="col">Horario</th><th scope="col">Estado</th><th scope="col">Acciones</th></tr>
              </thead>
              <tbody>
                {exceptions.map((row) => {
                  const exception = row.accepted?.state ?? row.provisional?.state ?? null;
                  if (!exception) return null;
                  const busy = row.provisional !== null;
                  return (
                    <tr key={exception.exceptionId}>
                      <td className="num">{exception.localDate}</td>
                      <td>{exception.kind === "closed" ? "Cierre" : "Reemplazo"}</td>
                      <td className="num">
                        {exception.kind === "closed" ? "—" : `${minutesToTime(exception.localStartMinute ?? 0)}–${minutesToTime(exception.localEndMinute ?? 0)}`}
                      </td>
                      <td><span className="badge">{exception.status === "active" ? "Activa" : "Inactiva"}</span></td>
                      <td>
                        <button type="button" className="btn btn--subtle" disabled={busy || saving}
                          onClick={() => void run(() => setExceptionStatus({
                            exceptionId: exception.exceptionId, base: baseOf(row),
                            status: exception.status === "active" ? "inactive" : "active",
                          }), `Excepción ${exception.status === "active" ? "desactivada" : "activada"}.`)}>
                          <Power size={14} aria-hidden="true" />
                          {exception.status === "active" ? "Desactivar" : "Activar"}
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {/* Bloqueos */}
      <section aria-labelledby="bloqueos-titulo" style={{ marginTop: 24 }}>
        <div className="page-header">
          <h2 id="bloqueos-titulo" style={{ fontSize: 18 }}>Bloqueos</h2>
          {editor !== "block" && (
            <button type="button" className="btn"
              onClick={() => { setEditor("block"); setBlockDraft({ start: "", end: "", reason: "vacation", global: false }); }}>
              <Plus size={16} aria-hidden="true" />
              Nuevo bloqueo
            </button>
          )}
        </div>
        {editor === "block" && (
          <form className="card" style={{ padding: 16, display: "grid", gap: 12, marginBottom: 12 }} onSubmit={(e) => void submitBlock(e)}>
            <p className="muted" style={{ fontSize: 13 }}>
              Un bloqueo impide reservar en un intervalo concreto. El bloqueo global solo lo gestiona el
              profesional propio; el proceso local valida el permiso.
            </p>
            <div className="toolbar">
              <label className="field-label">
                Inicio
                <input type="datetime-local" className="input-date" value={blockDraft.start}
                  onChange={(e) => setBlockDraft({ ...blockDraft, start: e.target.value })} />
              </label>
              <label className="field-label">
                Fin
                <input type="datetime-local" className="input-date" value={blockDraft.end}
                  onChange={(e) => setBlockDraft({ ...blockDraft, end: e.target.value })} />
              </label>
              <label className="field-label">
                Motivo
                <select className="select" value={blockDraft.reason}
                  onChange={(e) => setBlockDraft({ ...blockDraft, reason: e.target.value })}>
                  <option value="vacation">Vacaciones</option>
                  <option value="administrative">Administrativo</option>
                  <option value="unavailable">No disponible</option>
                </select>
              </label>
            </div>
            <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <input type="checkbox" checked={blockDraft.global}
                onChange={(e) => setBlockDraft({ ...blockDraft, global: e.target.checked })} />
              <span style={{ fontSize: 14 }}>Bloqueo global (toda la cuenta, solo profesional propio)</span>
            </label>
            <div className="toolbar">
              <button type="submit" className="btn btn--primary" disabled={saving}>Guardar bloqueo</button>
              <button type="button" className="btn" onClick={() => setEditor("none")}>Cancelar</button>
            </div>
          </form>
        )}
        {!blocks || blocks.length === 0 ? <p className="muted">Sin bloqueos registrados.</p> : (
          <div className="table-scroll">
            <table className="data-table">
              <thead>
                <tr><th scope="col">Intervalo</th><th scope="col">Ámbito</th><th scope="col">Motivo</th><th scope="col">Estado</th><th scope="col">Acciones</th></tr>
              </thead>
              <tbody>
                {blocks.map((row) => {
                  const block = row.accepted?.state ?? row.provisional?.state ?? null;
                  if (!block) return null;
                  const busy = row.provisional !== null;
                  return (
                    <tr key={block.blockId}>
                      <td className="num">{new Date(block.startsAt).toLocaleString("es")} – {new Date(block.endsAt).toLocaleString("es")}</td>
                      <td>{block.blockAccountId ? "Cuenta" : "Global"}</td>
                      <td>{block.reasonCode ? (blockReasonLabel[block.reasonCode] ?? block.reasonCode) : "No registrado"}</td>
                      <td><span className="badge">{block.status === "active" ? "Activo" : "Inactivo"}</span></td>
                      <td>
                        <button type="button" className="btn btn--subtle" disabled={busy || saving}
                          onClick={() => void run(() => setBlockStatus({
                            blockId: block.blockId, base: baseOf(row),
                            status: block.status === "active" ? "inactive" : "active",
                          }), `Bloqueo ${block.status === "active" ? "desactivado" : "activado"}.`)}>
                          <Power size={14} aria-hidden="true" />
                          {block.status === "active" ? "Desactivar" : "Activar"}
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </>
  );
}
