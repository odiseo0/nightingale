import { useCallback, useEffect, useRef, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ArrowRight, CircleNotch, SignIn, SignOut, X } from "@phosphor-icons/react";
import { getAccountView } from "../../shared/api/account";
import {
  activateAccount, beginLogin, cancelLogin, completeLogin, getCoreStatus, listAccounts, logout,
  type AccountChoice, type CoreStatus,
} from "../../shared/api/identity";
import {
  getBookingContext, type BookingContext,
} from "../../shared/api/appointments";
import { listRules } from "../../shared/api/scheduling";
import { CheckCircle, Warning } from "@phosphor-icons/react";

type Phase = "loading" | "ready" | "error";

/** S01 · Acceso y preparación: flujo de acceso externo real, selección de
 * cuenta y preparación derivada de datos descargados. */
export default function AccessPage() {
  const navigate = useNavigate();
  const [phase, setPhase] = useState<Phase>("loading");
  const [core, setCore] = useState<CoreStatus | null>(null);
  const [accounts, setAccounts] = useState<AccountChoice[]>([]);
  const [accountLabel, setAccountLabel] = useState<string | null>(null);
  const [context, setContext] = useState<BookingContext | null>(null);
  const [ruleCount, setRuleCount] = useState<number | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const generation = useRef(0);

  const refresh = useCallback(async () => {
    const request = ++generation.current;
    try {
      const [status, screen] = await Promise.all([getCoreStatus(), getAccountView()]);
      if (request !== generation.current) return;
      setCore(status);
      setAccountLabel(screen.account?.displayName ?? null);
      if (status.signedIn && !status.accountId) {
        const choices = await listAccounts();
        if (request === generation.current) setAccounts(choices);
      }
      if (status.accountId) {
        const [booking, rules] = await Promise.all([getBookingContext(), listRules()]);
        if (request !== generation.current) return;
        setContext(booking);
        setRuleCount(rules.length);
      }
      setPhase("ready");
    } catch {
      if (request === generation.current) setPhase("error");
    }
  }, []);

  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => void refresh(), 5000);
    return () => { generation.current += 1; window.clearInterval(timer); };
  }, [refresh]);

  async function act(name: string, work: () => Promise<unknown>, success?: string) {
    setBusy(name);
    setMessage(null);
    try {
      await work();
      if (success) setMessage(success);
      await refresh();
    } catch {
      setMessage("No se pudo completar la acción. Comprueba el acceso, la configuración y la conexión desde la aplicación de escritorio.");
    } finally {
      setBusy(null);
    }
  }

  if (phase === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Comprobando el acceso local…</p>
      </div>
    );
  }

  if (phase === "error") {
    return (
      <div className="state-block" role="alert">
        <p>No se pudo leer el estado de acceso local.</p>
        <button type="button" className="btn" onClick={() => void refresh()}>Reintentar</button>
      </div>
    );
  }

  const ready = core?.signedIn && core.accountId && core.access === "active";

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">Acceso y preparación</h1>
          <p className="muted">
            {ready
              ? `Acceso activo${accountLabel ? ` · ${accountLabel}` : ""}. Los datos locales y los pendientes se conservan.`
              : "Activa una cuenta autorizada para trabajar con los datos de esta consulta."}
          </p>
        </div>
        {ready && (
          <button type="button" className="btn btn--primary" onClick={() => navigate("/agenda")}>
            Ir a la agenda
            <ArrowRight size={16} aria-hidden="true" />
          </button>
        )}
      </div>

      {message && <div className="notice" role="status"><p>{message}</p></div>}

      {/* Estados de acceso */}
      {core && !core.configured && (
        <div className="notice" role="alert">
          <p>El servicio remoto no está configurado en este equipo. El diagnóstico sigue disponible en Desarrollo.</p>
        </div>
      )}
      {core?.signedIn && core.access === "revoked" && (
        <div className="notice" role="alert">
          <Warning size={18} aria-hidden="true" />
          <p>El acceso fue revocado. Los datos y pendientes se conservan; al reconectar se aplicará la política de acceso vigente.</p>
        </div>
      )}
      {core?.signedIn && core.access === "inactive" && (
        <div className="notice" role="alert">
          <Warning size={18} aria-hidden="true" />
          <p>La cuenta está inactiva. Revisa la configuración de la cuenta cuando tengas conexión.</p>
        </div>
      )}
      {core?.outcome === "offline" && (
        <div className="notice">
          <p>Sin conexión: trabajas con los últimos permisos válidos. Los cambios de permiso requieren conexión.</p>
        </div>
      )}
      {core?.outcome === "recovery_required" && (
        <div className="notice" role="alert">
          <Warning size={18} aria-hidden="true" />
          <p>Se requiere recuperar la vista local. Conéctate para recuperarla; los datos locales y pendientes no se borran.</p>
        </div>
      )}

      {/* Acceso */}
      <section className="card" style={{ padding: 16 }} aria-labelledby="acceso-titulo">
        <h2 id="acceso-titulo" style={{ fontSize: 18, marginBottom: 12 }}>Acceso</h2>
        {!core?.signedIn ? (
          <div className="toolbar">
            <button
              type="button"
              className="btn btn--primary"
              disabled={!!busy || !core?.configured}
              onClick={() => void act("login", async () => {
                await beginLogin();
                setAccounts(await completeLogin());
              })}
            >
              <SignIn size={16} aria-hidden="true" />
              Iniciar acceso
            </button>
            {busy === "login" && (
              <button type="button" className="btn" onClick={() => void act("cancel", () => cancelLogin())}>
                <X size={16} aria-hidden="true" />
                Cancelar acceso
              </button>
            )}
            <p className="muted" role="status">
              {busy === "login"
                ? "Completa el acceso en el navegador externo que se abrió."
                : "El acceso se completa en tu navegador con la cuenta autorizada."}
            </p>
          </div>
        ) : (
          <div className="toolbar">
            <p className="muted">
              {core.accountId
                ? `Cuenta activa${accountLabel ? `: ${accountLabel}` : ""}.`
                : "Sesión iniciada. Elige la cuenta de esta consulta."}
            </p>
            {core.accountId && (
              <button
                type="button"
                className="btn"
                disabled={!!busy}
                onClick={() => void act("logout", async () => { await logout(); setAccounts([]); setContext(null); })}
              >
                <SignOut size={16} aria-hidden="true" />
                Cerrar sesión local
              </button>
            )}
          </div>
        )}

        {accounts.length > 0 && (
          <ul style={{ margin: "12px 0 0", paddingLeft: 20, display: "grid", gap: 8 }}>
            {accounts.map((account) => (
              <li key={account.id}>
                <button
                  type="button"
                  className="btn"
                  disabled={!!busy || account.status !== "active"}
                  onClick={() => void act("activate", () => activateAccount(account.id), "Cuenta activada. Preparando los datos iniciales.")}
                >
                  {account.displayName}
                  {account.id === core?.accountId ? " · Actual" : ""}
                  {account.status !== "active" && " (inactiva)"}
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* Preparación */}
      {ready && (
        <section className="card" style={{ padding: 16 }} aria-labelledby="preparacion-titulo">
          <h2 id="preparacion-titulo" style={{ fontSize: 18, marginBottom: 12 }}>Preparación de la consulta</h2>
          <ul style={{ margin: 0, paddingLeft: 20, display: "grid", gap: 8 }}>
            <PreparationStep
              ok={Boolean(accountLabel)}
              title={`Cuenta y zona${context ? ` (${context.timeZone})` : ""}`}
              detail={accountLabel ? `Cuenta activa: ${accountLabel}` : "Sin datos de cuenta descargados."}
            />
            <PreparationStep
              ok={(context?.professionals.length ?? 0) > 0}
              title="Perfil profesional"
              detail={`${context?.professionals.length ?? 0} profesional(es) en el contexto local.`}
            />
            <PreparationStep
              ok={(context?.offers.length ?? 0) > 0}
              title="Servicios y ofertas"
              detail={`${context?.offers.length ?? 0} oferta(s) de servicio disponibles.`}
            />
            <PreparationStep
              ok={(ruleCount ?? 0) > 0}
              title="Horario semanal"
              detail={ruleCount === null ? "Sin datos de horario descargados." : `${ruleCount} regla(s) de horario activas.`}
            />
          </ul>
          <p className="muted" style={{ fontSize: 13, marginTop: 12 }}>
            La gestión detallada de cuenta, servicios y horarios vive en Configuración (planes
            siguientes); mientras tanto puedes revisarla en{" "}
            <Link to="/desarrollo">Desarrollo</Link>. No hace falta completar nada más para abrir la agenda.
          </p>
        </section>
      )}
    </>
  );
}

function PreparationStep({ ok, title, detail }: { ok: boolean; title: string; detail: string }) {
  return (
    <li style={{ display: "flex", gap: 8, alignItems: "flex-start" }}>
      {ok
        ? <CheckCircle size={18} aria-hidden="true" style={{ color: "var(--color-status-success)", flexShrink: 0, marginTop: 2 }} />
        : <Warning size={18} aria-hidden="true" style={{ color: "var(--color-text-tertiary)", flexShrink: 0, marginTop: 2 }} />}
      <div>
        <p style={{ fontWeight: 600 }}>{title}</p>
        <p className="muted" style={{ fontSize: 13 }}>{detail}</p>
      </div>
    </li>
  );
}
