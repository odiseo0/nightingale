import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  getAccountView, setPreference, updateAccountDetails, updateAccountPolicy, setAccountStatus,
  type AccountDetailsPatch, type AccountMutationResult, type AccountScreen, type Theme,
} from "../../shared/api/account";

const detailFields = [
  ["displayName", "Nombre de la cuenta"], ["timeZone", "Zona IANA"],
  ["locationName", "Nombre del lugar"], ["locationAddress", "Dirección"],
  ["locationCity", "Ciudad"], ["locationState", "Estado o provincia"],
  ["locationCountry", "País (ISO alfa-2)"],
] as const;

function outcome(result: AccountMutationResult): string {
  if (result.status === "accepted") {
    return result.refreshStatus === "failed"
      ? "La API aceptó el cambio. Falta descargar la vista actual; sincroniza antes de repetir."
      : "La API aceptó el cambio. La vista muestra los últimos datos descargados.";
  }

  return `El cambio no se aceptó: ${result.code ?? result.status}. Revisa la cuenta antes de repetir.`;
}

export default function SettingsPanel() {
  const [screen, setScreen] = useState<AccountScreen | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const generation = useRef(0);

  // The cleanup and request number prevent a late read from restoring an old
  // screen after a newer read or unmount. Reads use local IPC, never HTTP.
  // https://react.dev/reference/react/useEffect#fetching-data-with-effects
  useEffect(() => {
    const refresh = async () => {
      const request = ++generation.current;

      try {
        const view = await getAccountView();
        if (request === generation.current) setScreen(view);
      } catch {
        if (request === generation.current) setScreen(null);
      }
    };

    void refresh();
    const timer = window.setInterval(() => void refresh(), 2000);
    return () => { ++generation.current; window.clearInterval(timer); };
  }, []);

  async function act(work: () => Promise<AccountMutationResult | void>) {
    setBusy(true);
    setMessage("");

    try {
      const result = await work();
      setMessage(result ? outcome(result) : "Preferencias guardadas en este dispositivo.");
      const request = ++generation.current;
      const view = await getAccountView();

      if (request === generation.current) setScreen(view);
    } catch (error: unknown) {
      // Use safe Rust codes; do not display arbitrary transport/error objects.
      const code = typeof error === "object" && error !== null && "code" in error ? String(error.code) : "TEMPORARY_FAILURE";
      setMessage(`No se pudo completar la acción (${code}). Revisa acceso y conexión. Si el resultado es desconocido, sincroniza antes de repetir.`);
    } finally { setBusy(false); }
  }

  function saveDetails(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!screen?.account) return;

    const account = screen.account;
    const data = new FormData(event.currentTarget);
    const patch: AccountDetailsPatch = {};

    for (const [field] of detailFields) {
      const value = String(data.get(field) ?? "");

      if (value !== account[field]) patch[field] = value;
    }

    void act(() => updateAccountDetails({ expectedVersion: account.version, patch }));
  }

  function savePolicy(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!screen?.account) return;
    const data = new FormData(event.currentTarget);
    void act(() => updateAccountPolicy({ expectedVersion: screen.account!.version, noShowGraceMinutes: Number(data.get("noShowGraceMinutes")) }));
  }

  function savePreferences(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const theme = String(data.get("theme")) as Theme;
    const showInactive = data.has("servicesShowInactive");
    void act(async () => {
      await setPreference({ key: "theme", value: theme });
      await setPreference({ key: "servicesShowInactive", value: showInactive });
    });
  }

  const account = screen?.account;
  return <section aria-labelledby="settings-heading" className="space-y-4 border-t border-gray-200 pt-6">
    <h2 id="settings-heading" className="text-xl font-semibold">Cuenta y preferencias</h2>
    {!screen && <p>Inicia sesión y elige una cuenta con acceso activo para consultar sus datos.</p>}
    {screen && !account && <p>Aún no hay una vista de cuenta descargada. Sincroniza para obtenerla.</p>}
    {account && <>
      <dl className="grid gap-2 sm:grid-cols-2">
        <div><dt>Cuenta</dt><dd>{account.displayName}</dd></div>
        <div><dt>Zona horaria</dt><dd>{account.timeZone}</dd></div>
        <div><dt>Lugar</dt><dd>{account.locationName}, {account.locationAddress}, {account.locationCity}, {account.locationState}, {account.locationCountry}</dd></div>
        <div><dt>Tolerancia por ausencia</dt><dd>{account.policy.noShowGraceMinutes} minutos</dd></div>
        <div><dt>Estado</dt><dd>{account.status === "active" ? "Activa" : "Inactiva"}</dd></div>
        <div><dt>Versión aceptada</dt><dd>{account.version}</dd></div>
      </dl>
      {screen.canManage ? <>
        <p>Los cambios de cuenta requieren conexión y permiso de propietario.</p>
        <form key={`details-${account.accountId}-${account.version}`} onSubmit={saveDetails}>
          <fieldset disabled={busy} className="grid gap-3 sm:grid-cols-2">
            <legend>Datos de la cuenta</legend>
            {detailFields.map(([field, label]) => <label key={field} className="grid gap-1">{label}<input name={field} defaultValue={account[field]} required className="rounded border p-2" /></label>)}
            <button type="submit">Guardar datos con conexión</button>
          </fieldset>
        </form>
        <form key={`policy-${account.accountId}-${account.version}`} onSubmit={savePolicy}>
          <fieldset disabled={busy} className="space-y-3">
            <legend>Política de la cuenta</legend>
            <label className="grid gap-1">Tolerancia por ausencia (minutos)<input name="noShowGraceMinutes" type="number" min="0" max="1440" step="1" required defaultValue={account.policy.noShowGraceMinutes} className="rounded border p-2" /></label>
            <button type="submit">Guardar política con conexión</button>
          </fieldset>
        </form>
        <button disabled={busy} onClick={() => void act(() => setAccountStatus({ expectedVersion: account.version, status: account.status === "active" ? "inactive" : "active" }))}>{account.status === "active" ? "Desactivar cuenta con conexión" : "Activar cuenta con conexión"}</button>
      </> : <p>Los datos de cuenta son de solo lectura para este acceso.</p>}
    </>}
    {screen && <form key={`preferences-${account?.accountId ?? "selected"}-${screen.preferences.theme}-${screen.preferences.servicesShowInactive}`} onSubmit={savePreferences}>
      <fieldset disabled={busy} className="space-y-3">
        <legend>Preferencias de este dispositivo</legend>
        <label className="grid gap-1">Tema<select name="theme" defaultValue={screen.preferences.theme} className="rounded border p-2"><option value="system">Sistema</option><option value="light">Claro</option><option value="dark">Oscuro</option></select></label>
        <label className="flex gap-2"><input type="checkbox" name="servicesShowInactive" defaultChecked={screen.preferences.servicesShowInactive} />Mostrar servicios inactivos</label>
        <button type="submit">Guardar preferencias locales</button>
      </fieldset>
    </form>}
    {message && <p role="status">{message}</p>}
  </section>;
}
