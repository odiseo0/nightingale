import { useEffect, useRef, useState } from "react";
import { NavLink, Outlet, useLocation } from "react-router-dom";
import { CalendarBlank, Gear, List, MagnifyingGlass, Question, Users, ArrowsClockwise, SignIn, X } from "@phosphor-icons/react";
import CommandPalette from "../features/search/CommandPalette";
import { getAccountView, type AccountView } from "../shared/api/account";
import { getCoreStatus } from "../shared/api/identity";

const TITLES: [RegExp, string][] = [
  [/^\/agenda/, "Agenda"],
  [/^\/pacientes\/nueva/, "Registrar paciente"],
  [/^\/pacientes/, "Pacientes"],
  [/^\/paciente\/.*\/editar/, "Editar paciente"],
  [/^\/paciente\//, "Ficha del paciente"],
  [/^\/cita\/nueva/, "Nueva cita"],
  [/^\/cita\/.*\/reprogramar/, "Reprogramar cita"],
  [/^\/cita\//, "Detalle de cita"],
  [/^\/cambios/, "Cambios"],
  [/^\/acceso/, "Acceso y preparación"],
  [/^\/diagnostico/, "Diagnóstico"],
  [/^\/configuracion/, "Configuración"],
  [/^\/desarrollo/, "Desarrollo"],
];

function titleFor(pathname: string): string {
  for (const [pattern, title] of TITLES) {
    if (pattern.test(pathname)) return title;
  }
  return "Agenda";
}

export default function AppLayout() {
  const location = useLocation();
  const [account, setAccount] = useState<AccountView | null>(null);
  const [attentionCount, setAttentionCount] = useState(0);
  const [signedIn, setSignedIn] = useState(true);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);
  const paletteTriggerRef = useRef<HTMLButtonElement>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const menuToggleRef = useRef<HTMLButtonElement>(null);
  const [zone, setZone] = useState("UTC");

  useEffect(() => {
    getAccountView()
      .then((screen) => { if (screen.account) setZone(screen.account.timeZone); })
      .catch(() => { /* zona por defecto */ });
  }, [location.pathname]);

  /* Ctrl/Cmd+K abre la búsqueda global (con alternativa visible en la barra). */
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPaletteOpen((open) => !open);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  /* Enlace discreto al centro de cambios: solo con contador cuando hay algo. */
  useEffect(() => {
    let cancelled = false;
    const refresh = () => getCoreStatus()
      .then((status) => {
        if (cancelled) return;
        setSignedIn(status.signedIn);
        const outbox = status.sync?.outbox;
        setAttentionCount(outbox
          ? outbox.pending + outbox.retryable + outbox.unknown + outbox.inFlight
            + outbox.conflicts + outbox.rejected + outbox.reviewRequired
          : 0);
      })
      .catch(() => { /* sin sesión: sin contador */ });
    void refresh();
    const timer = window.setInterval(() => void refresh(), 15_000);
    return () => { cancelled = true; window.clearInterval(timer); };
  }, [location.pathname]);

  useEffect(() => {
    let cancelled = false;
    getAccountView()
      .then((screen) => {
        if (!cancelled) setAccount(screen.account);
      })
      .catch(() => { /* contexto de cuenta no disponible */ });
    return () => { cancelled = true; };
  }, [location.pathname]);

  useEffect(() => {
    setMenuOpen(false);
  }, [location.pathname]);

  useEffect(() => {
    if (!menuOpen) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setMenuOpen(false);
        menuToggleRef.current?.focus();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [menuOpen]);

  const title = titleFor(location.pathname);
  const accountLine = account
    ? `${account.displayName} · ${account.locationName}`.trim()
    : "Sin cuenta activa";

  return (
    <div className="app-shell">
      <a href="#contenido" className="skip-link">
        Ir al contenido
      </a>
      <button
        ref={menuToggleRef}
        type="button"
        className="menu-toggle"
        aria-expanded={menuOpen}
        aria-controls="navegacion-principal"
        aria-label={menuOpen ? "Cerrar navegación" : "Abrir navegación"}
        onClick={() => setMenuOpen((open) => !open)}
      >
        {menuOpen ? <X size={20} /> : <List size={20} />}
      </button>

      <aside className="app-sidebar" data-open={menuOpen} id="navegacion-principal">
        <div className="app-brand">Nightingale</div>
        <nav className="app-nav" aria-label="Navegación principal">
          <NavLink to="/agenda" className="nav-link">
            <CalendarBlank size={18} aria-hidden="true" />
            Agenda
          </NavLink>
          {!signedIn && (
            <NavLink to="/acceso" className="nav-link">
              <SignIn size={18} aria-hidden="true" />
              Acceso
            </NavLink>
          )}
          <NavLink to="/pacientes" className="nav-link">
            <Users size={18} aria-hidden="true" />
            Pacientes
          </NavLink>
          <NavLink to="/cambios" className="nav-link">
            <ArrowsClockwise size={18} aria-hidden="true" />
            Cambios
            {attentionCount > 0 && (
              <span className="badge badge--selected" aria-label={`${attentionCount} cambios pendientes o por revisar`}>
                {attentionCount}
              </span>
            )}
          </NavLink>
          <NavLink to="/configuracion/cuenta" className="nav-link">
            <Gear size={18} aria-hidden="true" />
            Configuración
          </NavLink>
          <NavLink to="/desarrollo" className="nav-link">
            <List size={18} aria-hidden="true" />
            Desarrollo
          </NavLink>
        </nav>
      </aside>

      <div className="app-main">
        <header className="app-topbar">
          <div className="topbar-context">
            <span className="topbar-title">{title}</span>
            <span className="topbar-subtitle">{accountLine}</span>
          </div>
          <div className="toolbar" style={{ gap: 8 }}>
            <button
              ref={paletteTriggerRef}
              type="button"
              className="btn btn--subtle"
              onClick={() => setPaletteOpen(true)}
              aria-haspopup="dialog"
            >
              <MagnifyingGlass size={16} aria-hidden="true" />
              Buscar
              <span className="muted" style={{ fontSize: 12 }}>Ctrl+K</span>
            </button>
            <button type="button" className="icon-btn" aria-label="Ayuda" onClick={() => setHelpOpen(true)}>
              <Question size={18} aria-hidden="true" />
            </button>
          </div>
        </header>
        <main id="contenido" className="app-content">
          <Outlet />
        </main>
      </div>

      <CommandPalette
        open={paletteOpen}
        onClose={() => { setPaletteOpen(false); paletteTriggerRef.current?.focus(); }}
        zone={zone}
      />

      {helpOpen && (
        <div className="modal-backdrop" role="dialog" aria-modal="true" aria-label="Ayuda" onClick={() => setHelpOpen(false)}>
          <div className="modal-card" onClick={(event) => event.stopPropagation()}>
            <h2 style={{ fontSize: 20 }}>Ayuda</h2>
            <p className="muted">
              Estás en <strong>{title}</strong>. La navegación muestra solo superficies funcionales;
              el diagnóstico técnico vive fuera del trabajo diario en Desarrollo.
            </p>
            <ul style={{ margin: 0, paddingLeft: 20, display: "grid", gap: 6 }}>
              <li>Agenda: día, semana y lista de citas con detalle y acciones del ciclo.</li>
              <li>Pacientes: directorio, ficha administrativa y registro.</li>
              <li>Cambios: sincronización pendiente y conflictos.</li>
              <li>Configuración: cuenta, servicios y horarios.</li>
            </ul>
            <div className="toolbar">
              <NavLink to="/diagnostico" className="btn" onClick={() => setHelpOpen(false)}>
                Abrir diagnóstico (S13)
              </NavLink>
              <NavLink to="/desarrollo" className="btn" onClick={() => setHelpOpen(false)}>
                Panel técnico (Desarrollo)
              </NavLink>
              <button type="button" className="btn btn--primary" onClick={() => setHelpOpen(false)}>
                Cerrar
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
