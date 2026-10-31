import { useEffect, useState } from "react";
import { activateAccount, beginLogin, cancelLogin, completeLogin, getCoreStatus, listAccounts, logout, syncNow, type AccountChoice, type CoreStatus } from "./shared/api/identity";

export default function CorePanel() {
  const [view, setView] = useState<CoreStatus | null>(null);
  const [accounts, setAccounts] = useState<AccountChoice[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    let current = true;
    const refresh = () => getCoreStatus().then(value => { if (current) setView(value); }).catch(() => { if (current) setError(true); });
    void refresh();
    const timer = window.setInterval(() => void refresh(), 1000);
    return () => { current = false; window.clearInterval(timer); };
  }, []);

  async function act(name: string, work: () => Promise<unknown>) {
    setBusy(name); setError(false);
    try { await work(); setView(await getCoreStatus()); } catch { setError(true); } finally { setBusy(null); }
  }

  const outbox = view?.sync?.outbox;
  let label = "Sin sesión";
  if (view?.signedIn) {
    label = !view.accountId ? "Elige una cuenta" : "Estado local disponible";
    if (view.sync && view.sync.phase !== "idle") label = "Sincronizando";
    else if (view.access === "revoked" || view.access === "inactive" || view.outcome === "access_denied") label = "Acceso denegado";
    else if (view.outcome === "offline") label = "Sin conexión; se conserva el trabajo local";
    else if (view.outcome === "failed" || view.outcome === "retry_waiting") label = "Servicio no disponible; puedes reintentar";
    else if (outbox && (outbox.conflicts + outbox.rejected + outbox.reviewRequired + outbox.otherEpoch > 0)) label = "Hay operaciones que requieren revisión";
    else if (outbox && (outbox.pending + outbox.unknown + outbox.retryable + outbox.inFlight > 0)) label = "Hay operaciones pendientes";
    else if (view.outcome === "complete") label = "Descarga completada";
    else if (view.outcome === "recovery_required") label = "Se requiere recuperar la vista local";
  }

  return <section aria-labelledby="core-heading" className="space-y-4 border-t border-gray-200 pt-6">
    <h2 id="core-heading" className="text-xl font-semibold">Acceso y sincronización</h2>
    <p role="status">{busy === "login" ? "Completa el acceso en el navegador externo." : label}</p>
    {view && !view.configured && <p>La API de desarrollo aún no está configurada. El diagnóstico local sigue disponible.</p>}
    <div className="flex flex-wrap gap-3">
      {!view?.signedIn && <button disabled={!!busy || !view?.configured} onClick={() => void act("login", async () => { await beginLogin(); setAccounts(await completeLogin()); })}>Iniciar sesión</button>}
      {busy === "login" && <button onClick={() => void cancelLogin().catch(() => setError(true))}>Cancelar acceso</button>}
      {view?.signedIn && <>
        <button disabled={!!busy} onClick={() => void act("accounts", async () => setAccounts(await listAccounts()))}>Elegir cuenta</button>
        <button disabled={!!busy || !view.accountId || !view.configured} onClick={() => void act("sync", syncNow)}>Sincronizar ahora</button>
      </>}
      {view?.configured && <button disabled={!!busy} onClick={() => void act("logout", async () => { await logout(); setAccounts([]); })}>Cerrar sesión local</button>}
    </div>
    {accounts.length > 0 && <ul className="space-y-2">{accounts.map(account => <li key={account.id}><button disabled={!!busy || account.status !== "active"} onClick={() => void act("activate", async () => { await activateAccount(account.id); setAccounts([]); })}>{account.displayName}{account.id === view?.accountId ? " · Actual" : ""}</button></li>)}</ul>}
    {outbox && <dl className="grid gap-3 sm:grid-cols-3">
      <div><dt>Pendientes</dt><dd>{outbox.pending + outbox.retryable + outbox.unknown + outbox.inFlight}</dd></div>
      <div><dt>Conflictos</dt><dd>{outbox.conflicts}</dd></div>
      <div><dt>Rechazos y revisiones</dt><dd>{outbox.rejected + outbox.reviewRequired + outbox.otherEpoch}</dd></div>
    </dl>}
    {!!view?.reviews.length && <ul aria-label="Operaciones para revisión" className="space-y-2">{view.reviews.map(row => <li key={row.operationId}><code>{row.operationId}</code> · {row.status}{row.code ? ` · ${row.code}` : ""}</li>)}</ul>}
    <p>Las operaciones pendientes conservan su estado hasta que la API confirme el resultado.</p>
    {view?.lastSyncAt != null && <p>Último intercambio en esta ejecución (UTC): {new Date(view.lastSyncAt).toISOString()}</p>}
    {error && <p role="alert">No se pudo completar la acción. Comprueba el acceso, la configuración y la conexión desde la aplicación de escritorio.</p>}
  </section>;
}
