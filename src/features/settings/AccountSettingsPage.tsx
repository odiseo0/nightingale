import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";
import { useBlocker } from "react-router-dom";
import { CircleNotch, Lock } from "@phosphor-icons/react";
import Modal from "../../components/primitives/Modal";
import {
  getAccountView, getPreferences, setPreference, updateAccountDetails, updateAccountPolicy,
  type AccountScreen, type AccountView,
} from "../../shared/api/account";

/** S12 · Cuenta: datos compartidos (canal remoto) y preferencias del
 * dispositivo (canal local), separados. Solo tema claro implementado. */
export default function AccountSettingsPage() {
  const [screen, setScreen] = useState<AccountScreen | null>(null);
  const [phase, setPhase] = useState<"loading" | "ready" | "no-session" | "error">("loading");
  const [form, setForm] = useState<Record<string, string>>({});
  const [grace, setGrace] = useState("");
  const [showInactive, setShowInactive] = useState(false);
  const [saving, setSaving] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const savingRef = useRef(false);
  const savedRef = useRef(false);
  const dirtyRef = useRef(false);

  const load = useCallback(async () => {
    try {
      const [view, preferences] = await Promise.all([getAccountView(), getPreferences()]);
      setScreen(view);
      setShowInactive(preferences.servicesShowInactive);
      const account = view.account;
      if (account) {
        setForm({
          displayName: account.displayName,
          timeZone: account.timeZone,
          locationName: account.locationName,
          locationAddress: account.locationAddress,
          locationCity: account.locationCity,
          locationState: account.locationState,
          locationCountry: account.locationCountry,
        });
        setGrace(String(account.policy.noShowGraceMinutes));
      }
      setPhase(view.account ? "ready" : "no-session");
    } catch {
      setPhase("error");
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const account: AccountView | null = screen?.account ?? null;
  const original = account ? {
    displayName: account.displayName,
    timeZone: account.timeZone,
    locationName: account.locationName,
    locationAddress: account.locationAddress,
    locationCity: account.locationCity,
    locationState: account.locationState,
    locationCountry: account.locationCountry,
  } : null;
  const dirty = (original !== null && JSON.stringify(form) !== JSON.stringify(original))
    || (account !== null && grace !== String(account.policy.noShowGraceMinutes));
  dirtyRef.current = dirty;
  const blocker = useBlocker(() => dirtyRef.current && !savedRef.current);

  useEffect(() => {
    if (!dirty || savedRef.current) return;
    const onBeforeUnload = (event: BeforeUnloadEvent) => { event.preventDefault(); };
    window.addEventListener("beforeunload", onBeforeUnload);
    return () => window.removeEventListener("beforeunload", onBeforeUnload);
  }, [dirty]);

  async function saveDetails(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!account || savingRef.current) return;
    savingRef.current = true;
    setSaving("details");
    setFormError(null);
    setNotice(null);
    try {
      const messages: string[] = [];
      let currentVersion = account.version;
      const patch: Record<string, string> = {};
      for (const key of Object.keys(form)) {
        if (form[key] !== original![key as keyof typeof original]) patch[key] = form[key];
      }
      if (Object.keys(patch).length > 0) {
        const result = await updateAccountDetails({ expectedVersion: currentVersion, patch });
        messages.push(resultMessage(result.status, result.refreshStatus));
        if (typeof result.entityVersion === "number") currentVersion = result.entityVersion;
      }
      const newGrace = Number(grace);
      if (newGrace !== account.policy.noShowGraceMinutes) {
        const result = await updateAccountPolicy({ expectedVersion: currentVersion, noShowGraceMinutes: newGrace });
        messages.push(resultMessage(result.status, result.refreshStatus));
      }
      setNotice(messages.length > 0 ? messages.join(" ") : "Sin cambios que guardar.");
      savedRef.current = true;
      await load();
    } catch (error: unknown) {
      setFormError(errorMessageOf(error));
    } finally {
      savingRef.current = false;
      setSaving(null);
    }
  }

  async function toggleShowInactive(value: boolean) {
    setShowInactive(value);
    try {
      await setPreference({ key: "servicesShowInactive", value });
      setNotice("Preferencia del dispositivo guardada (no requiere conexión).");
    } catch {
      setNotice("No se pudo guardar la preferencia local.");
      setShowInactive(!value);
    }
  }

  if (phase === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Cargando la configuración de la cuenta…</p>
      </div>
    );
  }

  if (phase === "no-session") {
    return (
      <div className="state-block" role="alert">
        <p>No hay una cuenta activa en este equipo.</p>
        <p className="muted">Activa una cuenta para ver y editar su configuración.</p>
      </div>
    );
  }

  if (phase === "error" || !account || !screen) {
    return (
      <div className="state-block" role="alert">
        <p>No se pudo leer la configuración local de la cuenta.</p>
        <button type="button" className="btn" onClick={() => void load()}>Reintentar</button>
      </div>
    );
  }

  const canManage = screen.canManage;

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">Cuenta</h1>
          <p className="muted">
            Datos compartidos de la cuenta
            {!canManage && " · solo lectura (sin permiso de gestión)"}
          </p>
        </div>
      </div>

      {notice && <div className="notice" role="status"><p>{notice}</p></div>}
      {formError && <div className="notice" role="alert"><p>{formError}</p></div>}

      <div style={{ display: "grid", gap: 24, maxWidth: 640 }}>
      <form onSubmit={(event) => void saveDetails(event)} style={{ display: "grid", gap: 16 }}>
        <fieldset className="card" style={{ padding: 16, display: "grid", gap: 12, border: "1px solid var(--color-border)" }} disabled={!canManage || saving !== null}>
          <legend className="muted">Datos compartidos (requieren conexión y permiso)</legend>
          <label className="field-label">
            Nombre de la consulta
            <input type="text" className="input-text" value={form.displayName ?? ""}
              onChange={(e) => setForm({ ...form, displayName: e.target.value })} />
          </label>
          <div className="toolbar">
            <label className="field-label" style={{ flex: 1, minWidth: 160 }}>
              Zona horaria
              <input type="text" className="input-text" value={form.timeZone ?? ""}
                placeholder="America/Caracas"
                onChange={(e) => setForm({ ...form, timeZone: e.target.value })} />
            </label>
            <label className="field-label" style={{ flex: 1, minWidth: 160 }}>
              Nombre del lugar
              <input type="text" className="input-text" value={form.locationName ?? ""}
                onChange={(e) => setForm({ ...form, locationName: e.target.value })} />
            </label>
          </div>
          <label className="field-label">
            Dirección
            <input type="text" className="input-text" value={form.locationAddress ?? ""}
              onChange={(e) => setForm({ ...form, locationAddress: e.target.value })} />
          </label>
          <div className="toolbar">
            <label className="field-label" style={{ flex: 1, minWidth: 120 }}>
              Ciudad
              <input type="text" className="input-text" value={form.locationCity ?? ""}
                onChange={(e) => setForm({ ...form, locationCity: e.target.value })} />
            </label>
            <label className="field-label" style={{ flex: 1, minWidth: 120 }}>
              Estado/Provincia
              <input type="text" className="input-text" value={form.locationState ?? ""}
                onChange={(e) => setForm({ ...form, locationState: e.target.value })} />
            </label>
            <label className="field-label" style={{ width: 90 }}>
              País
              <input type="text" className="input-text" value={form.locationCountry ?? ""}
                onChange={(e) => setForm({ ...form, locationCountry: e.target.value })} />
            </label>
          </div>
          <label className="field-label" style={{ maxWidth: 220 }}>
            Margen de ausencia (minutos)
            <input type="number" className="input-text" min={0} value={grace}
              onChange={(e) => setGrace(e.target.value)} />
          </label>
          <div className="toolbar">
            <button type="submit" className="btn btn--primary" disabled={!dirty}>
              <Lock size={16} aria-hidden="true" />
              {saving === "details" ? "Guardando…" : "Guardar datos de cuenta"}
            </button>
            {!canManage && <p className="muted">Tu acceso permite solo lectura de estos datos.</p>}
            {canManage && <p className="muted">Estos cambios requieren conexión y permiso de gestión; aquí se muestra solo el resultado conocido.</p>}
          </div>
        </fieldset>
      </form>

      <fieldset className="card" style={{ padding: 16, display: "grid", gap: 12, border: "1px solid var(--color-border)" }}>
        <legend className="muted">Preferencias de este dispositivo (sin conexión)</legend>
        <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <input
            type="checkbox"
            checked={showInactive}
            onChange={(event) => void toggleShowInactive(event.target.checked)}
          />
          Mostrar servicios inactivos en el catálogo
        </label>
        <div>
          <p style={{ fontWeight: 600 }}>Tema</p>
          <p className="muted" style={{ fontSize: 14 }}>
            Claro (único tema implementado). Otras opciones llegarán con la decisión de diseño correspondiente.
          </p>
        </div>
      </fieldset>
      </div>

      <Modal open={blocker.state === "blocked"} onClose={() => blocker.reset?.()} alert labelledBy="descartar-cuenta">
        <h2 id="descartar-cuenta" style={{ fontSize: 20 }}>Cambios sin guardar</h2>
        <p className="muted">Si sales ahora, los cambios de la cuenta no se guardarán.</p>
        <div className="toolbar">
          <button type="button" className="btn btn--primary" onClick={() => blocker.reset?.()}>
            Seguir editando
          </button>
          <button type="button" className="btn" onClick={() => blocker.proceed?.()}>
            Descartar y salir
          </button>
        </div>
      </Modal>
    </>
  );
}

function resultMessage(
  status: "accepted" | "conflict" | "rejected" | "retryable_failure",
  refreshStatus: "requested" | "failed" | "not_requested",
): string {
  const base = status === "accepted"
    ? "Cambio aceptado."
    : status === "conflict"
      ? "La cuenta cambió desde otro acceso; vuelve a cargar e inténtalo de nuevo."
      : status === "rejected"
        ? "Cambio no aceptado; revisa el permiso de tu acceso."
        : "Fallo temporal al enviar; el cambio no se guardó. Inténtalo de nuevo.";
  const refresh = refreshStatus === "failed"
    ? " La vista no se actualizó; los datos locales se conservan y no se presentan como actualizados."
    : "";
  return base + refresh;
}

function errorMessageOf(error: unknown): string {
  const code = typeof error === "object" && error !== null && "code" in error
    ? String((error as { code: string }).code)
    : "TEMPORARY_FAILURE";
  if (code === "VERSION_CONFLICT") return "La base local cambió; vuelve a cargar e inténtalo de nuevo.";
  if (code === "PERMISSION_DENIED" || code === "AUTHENTICATION_REQUIRED") {
    return "Tu acceso no permite esta acción; inicia sesión con una cuenta autorizada.";
  }
  return `No se pudo completar la acción (${code}). Inténtalo de nuevo.`;
}
