import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";
import { CircleNotch, Lock, PencilSimple, Plus, Power } from "@phosphor-icons/react";
import {
  createService, listCatalog, offerService, setServiceOfferStatus, setServiceStatus,
  updateProfessionalProfile, updateService, updateServiceOffer,
  type CatalogScreen,
} from "../../shared/api/services";
import { getPreferences, setPreference } from "../../shared/api/account";
import { mutationErrorMessage } from "../appointments/presentation";

type Editor =
  | { kind: "none" }
  | { kind: "new-service" }
  | { kind: "edit-service"; serviceId: string; name: string; duration: string }
  | { kind: "offer"; professionalId: string; serviceId: string; override: string }
  | { kind: "edit-offer"; professionalId: string; serviceId: string; override: string }
  | { kind: "professional"; professionalId: string; publicName: string };

/** S12 · Servicios: catálogo del profesional y ofertas de la cuenta con
 * duraciones base/override/efectiva, versiones y activo/inactivo. */
export default function ServicesPage() {
  const [screen, setScreen] = useState<CatalogScreen | null>(null);
  const [phase, setPhase] = useState<"loading" | "ready" | "error">("loading");
  const [showInactive, setShowInactive] = useState(false);
  const [editor, setEditor] = useState<Editor>({ kind: "none" });
  const [newName, setNewName] = useState("");
  const [newDuration, setNewDuration] = useState("30");
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const savingRef = useRef(false);
  const generation = useRef(0);

  const load = useCallback(async () => {
    const request = ++generation.current;
    try {
      const [catalog, preferences] = await Promise.all([listCatalog(), getPreferences()]);
      if (request !== generation.current) return;
      setScreen(catalog);
      setShowInactive(preferences.servicesShowInactive);
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
      setEditor({ kind: "none" });
      await load();
    } catch (error: unknown) {
      setFormError(mutationErrorMessage(error));
    } finally {
      savingRef.current = false;
      setSaving(false);
    }
  }

  async function toggleShowInactive(value: boolean) {
    setShowInactive(value);
    try {
      await setPreference({ key: "servicesShowInactive", value });
    } catch { setShowInactive(!value); }
  }

  function submitService(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (editor.kind === "new-service") {
      const professional = screen?.professionals[0]?.accepted;
      if (!professional) return;
      void run(() => createService({
        professionalId: professional.professionalId,
        name: newName.trim(),
        defaultDurationMinutes: Number(newDuration),
      }), "Servicio guardado.");
    } else if (editor.kind === "edit-service") {
      const duration = Number(newDuration);
      void run(() => updateService({
        serviceId: editor.serviceId,
        expectedVersion: versionOfService(editor.serviceId),
        patch: { name: newName.trim(), defaultDurationMinutes: duration },
      }));
    }
  }

  function submitOffer(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (editor.kind === "offer") {
      void run(() => offerService({
        professionalId: editor.professionalId,
        serviceId: editor.serviceId,
        durationMinutesOverride: newDuration.trim() === "" ? null : Number(newDuration),
      }), "Oferta guardada.");
    } else if (editor.kind === "edit-offer") {
      void run(() => updateServiceOffer({
        professionalId: editor.professionalId,
        serviceId: editor.serviceId,
        expectedVersion: offerVersion(editor.professionalId, editor.serviceId),
        durationMinutesOverride: newDuration.trim() === "" ? null : Number(newDuration),
      }));
    }
  }

  function submitProfessional(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (editor.kind !== "professional") return;
    void run(() => updateProfessionalProfile({
      professionalId: editor.professionalId,
      expectedVersion: professionalVersion(editor.professionalId),
      publicName: newName.trim(),
    }));
  }

  function versionOfService(serviceId: string): number {
    return screen?.services.find((row) => row.accepted?.serviceId === serviceId)?.accepted?.version ?? 0;
  }
  function offerVersion(professionalId: string, serviceId: string): number {
    return screen?.offers.find((row) => row.accepted?.professionalId === professionalId && row.accepted?.serviceId === serviceId)?.accepted?.version ?? 0;
  }
  function professionalVersion(professionalId: string): number {
    return screen?.professionals.find((row) => row.accepted?.professionalId === professionalId)?.accepted?.version ?? 0;
  }

  if (phase === "loading") {
    return (
      <div className="state-block" role="status">
        <CircleNotch size={22} aria-hidden="true" />
        <p>Cargando el catálogo…</p>
      </div>
    );
  }

  if (phase === "error") {
    return (
      <div className="state-block" role="alert">
        <p>No se pudo leer el catálogo local.</p>
        <button type="button" className="btn" onClick={() => void load()}>Reintentar</button>
      </div>
    );
  }

  if (!screen) return null;
  const canManage = screen.canManage;
  const professional = screen.professionals[0]?.accepted ?? null;

  const visibleServices = screen.services.filter((row) =>
    showInactive || (row.accepted?.status ?? "active") === "active");
  const visibleOffers = screen.offers.filter((row) =>
    showInactive || (row.accepted?.status ?? "active") === "active");

  const offeredIds = new Set(screen.offers.map((row) => row.accepted?.serviceId ?? row.provisional?.serviceId));

  return (
    <>
      <div className="page-header">
        <div>
          <h1 className="page-heading">Servicios y ofertas</h1>
          <p className="muted">
            Catálogo del profesional y ofertas de esta cuenta
            {!canManage && " · solo lectura (sin permiso de gestión)"}
          </p>
        </div>
        <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <input type="checkbox" checked={showInactive} onChange={(e) => void toggleShowInactive(e.target.checked)} />
          <span className="muted" style={{ fontSize: 14 }}>Mostrar inactivos</span>
        </label>
      </div>

      {notice && <div className="notice" role="status"><p>{notice}</p></div>}
      {formError && <div className="notice" role="alert"><p>{formError}</p></div>}

      {professional && (
        <section className="card" style={{ padding: 16 }} aria-labelledby="profesional-titulo">
          <h2 id="profesional-titulo" style={{ fontSize: 18, marginBottom: 8 }}>Perfil profesional</h2>
          {editor.kind === "professional" ? (
            <form className="toolbar" onSubmit={(e) => void submitProfessional(e)}>
              <label className="field-label">
                Nombre público
                <input type="text" className="input-text" value={newName}
                  onChange={(e) => setNewName(e.target.value)} />
              </label>
              <button type="submit" className="btn btn--primary" disabled={saving}>Guardar</button>
              <button type="button" className="btn" onClick={() => setEditor({ kind: "none" })}>Cancelar</button>
            </form>
          ) : (
            <div className="toolbar">
              <p>{professional.publicName} · {professional.status === "active" ? "Activo" : "Inactivo"}</p>
              {canManage && (
                <button type="button" className="btn btn--subtle"
                  onClick={() => { setEditor({ kind: "professional", professionalId: professional.professionalId, publicName: professional.publicName }); setNewName(professional.publicName); }}>
                  <PencilSimple size={16} aria-hidden="true" />
                  Editar nombre
                </button>
              )}
            </div>
          )}
        </section>
      )}

      <section aria-labelledby="catalogo-titulo">
        <div className="page-header">
          <h2 id="catalogo-titulo" style={{ fontSize: 18 }}>Catálogo de servicios</h2>
          {canManage && editor.kind !== "new-service" && (
            <button type="button" className="btn btn--primary"
              onClick={() => { setEditor({ kind: "new-service" }); setNewName(""); setNewDuration("30"); }}>
              <Plus size={16} aria-hidden="true" />
              Nuevo servicio
            </button>
          )}
        </div>

        {(editor.kind === "new-service" || editor.kind === "edit-service") && (
          <form className="card" style={{ padding: 16, display: "grid", gap: 12, marginBottom: 12 }}
            onSubmit={(e) => void submitService(e)}>
            <p className="muted" style={{ fontSize: 13 }}>
              <Lock size={14} aria-hidden="true" style={{ verticalAlign: -2 }} />{" "}
              Cambiar un servicio no modifica las citas anteriores: conservan sus datos guardados.
            </p>
            <div className="toolbar">
              <label className="field-label" style={{ flex: 1, minWidth: 200 }}>
                Nombre del servicio
                <input type="text" className="input-text" value={newName}
                  onChange={(e) => setNewName(e.target.value)} />
              </label>
              <label className="field-label" style={{ width: 120 }}>
                Duración base (min)
                <input type="number" className="input-text" min={5} value={newDuration}
                  onChange={(e) => setNewDuration(e.target.value)} />
              </label>
            </div>
            <div className="toolbar">
              <button type="submit" className="btn btn--primary" disabled={saving || !newName.trim()}>
                {saving ? "Guardando…" : "Guardar servicio"}
              </button>
              <button type="button" className="btn" onClick={() => setEditor({ kind: "none" })}>Cancelar</button>
            </div>
          </form>
        )}

        {visibleServices.length === 0 ? (
          <p className="muted">Sin servicios en los datos disponibles.</p>
        ) : (
          <div className="table-scroll">
            <table className="data-table">
              <thead>
                <tr>
                  <th scope="col">Servicio</th>
                  <th scope="col">Duración base</th>
                  <th scope="col">Estado</th>
                  <th scope="col">Acciones</th>
                </tr>
              </thead>
              <tbody>
                {visibleServices.map((row) => {
                  const service = row.accepted;
                  const draft = row.provisional;
                  const name = service?.name ?? draft?.name ?? "Servicio sin nombre";
                  const duration = service?.defaultDurationMinutes ?? draft?.defaultDurationMinutes ?? 0;
                  const status = service?.status ?? "active";
                  const busy = row.isProvisional;
                  return (
                    <tr key={service?.serviceId ?? `provisional-${name}`}>
                      <td>{name}</td>
                      <td className="num">{duration} min</td>
                      <td><span className="badge">{status === "active" ? "Activo" : "Inactivo"}</span></td>
                      <td>
                        {canManage && (
                          <div className="toolbar" style={{ gap: 6 }}>
                            <button type="button" className="btn btn--subtle" disabled={busy || saving}
                              onClick={() => { setEditor({ kind: "edit-service", serviceId: service?.serviceId ?? "", name, duration: String(duration) }); setNewName(name); setNewDuration(String(duration)); }}>
                              Editar
                            </button>
                            {!busy && service && !offeredIds.has(service.serviceId) && professional && (
                              <button type="button" className="btn btn--subtle" disabled={saving}
                                onClick={() => { setEditor({ kind: "offer", professionalId: professional.professionalId, serviceId: service.serviceId, override: "" }); setNewDuration(""); }}>
                                Ofrecer en esta cuenta
                              </button>
                            )}
                            <button type="button" className="btn btn--subtle" disabled={busy || saving}
                              onClick={() => service && void run(() => setServiceStatus({
                                serviceId: service.serviceId,
                                expectedVersion: service.version,
                                status: status === "active" ? "inactive" : "active",
                              }), `Servicio ${status === "active" ? "desactivado" : "activado"}.`)}>
                              <Power size={14} aria-hidden="true" />
                              {status === "active" ? "Desactivar" : "Activar"}
                            </button>
                          </div>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {(editor.kind === "offer" || editor.kind === "edit-offer") && (
        <form className="card" style={{ padding: 16, display: "grid", gap: 12, margin: "12px 0" }}
          onSubmit={(e) => void submitOffer(e)}>
          <p className="muted" style={{ fontSize: 13 }}>
            Ajuste de duración de la oferta en esta cuenta (vacío = duración base del servicio).
            Las citas existentes conservan su duración guardada.
          </p>
          <label className="field-label" style={{ width: 160 }}>
            Duración de la oferta (min, opcional)
            <input type="number" className="input-text" min={5} value={newDuration}
              onChange={(e) => setNewDuration(e.target.value)} />
          </label>
          <div className="toolbar">
            <button type="submit" className="btn btn--primary" disabled={saving}>Guardar oferta</button>
            <button type="button" className="btn" onClick={() => setEditor({ kind: "none" })}>Cancelar</button>
          </div>
        </form>
      )}

      <section aria-labelledby="ofertas-titulo" style={{ marginTop: 20 }}>
        <h2 id="ofertas-titulo" style={{ fontSize: 18, marginBottom: 8 }}>Ofertas de esta cuenta</h2>
        {visibleOffers.length === 0 ? (
          <p className="muted">Sin ofertas en los datos disponibles.</p>
        ) : (
          <div className="table-scroll">
            <table className="data-table">
              <thead>
                <tr>
                  <th scope="col">Servicio</th>
                  <th scope="col">Base / Ajuste / Efectiva</th>
                  <th scope="col">Estado</th>
                  <th scope="col">Acciones</th>
                </tr>
              </thead>
              <tbody>
                {visibleOffers.map((row) => {
                  const offer = row.accepted;
                  const draft = row.provisional;
                  const name = offer?.name ?? "Oferta sin nombre";
                  const base = offer?.defaultDurationMinutes ?? 0;
                  const override = offer?.durationMinutesOverride ?? draft?.durationMinutesOverride ?? null;
                  const effective = offer?.effectiveDurationMinutes
                    ?? (override ?? base);
                  const status = offer?.status ?? "active";
                  const serviceStatus = offer?.serviceStatus ?? "active";
                  const busy = row.isProvisional;
                  return (
                    <tr key={offer ? `${offer.professionalId}:${offer.serviceId}` : `provisional-${name}`}>
                      <td>{name}</td>
                      <td className="num">
                        {base} / {override ?? "—"} / {effective} min
                      </td>
                      <td>
                        <span className="badge">{status === "active" ? "Activo" : "Inactivo"}</span>{" "}
                        <span className={`badge ${serviceStatus === "active" ? "" : "badge--local"}`}>
                          servicio {serviceStatus === "active" ? "activo" : "inactivo"}
                        </span>
                      </td>
                      <td>
                        {canManage && (
                          <div className="toolbar" style={{ gap: 6 }}>
                            <button type="button" className="btn btn--subtle" disabled={busy || saving}
                              onClick={() => { if (!offer) return; setEditor({ kind: "edit-offer", professionalId: offer.professionalId, serviceId: offer.serviceId, override: String(override ?? "") }); setNewDuration(String(override ?? "")); }}>
                              Ajustar duración
                            </button>
                            <button type="button" className="btn btn--subtle" disabled={busy || saving}
                              onClick={() => offer && void run(() => setServiceOfferStatus({
                                professionalId: offer.professionalId,
                                serviceId: offer.serviceId,
                                expectedVersion: offer.version,
                                status: status === "active" ? "inactive" : "active",
                              }), `Oferta ${status === "active" ? "desactivada" : "activada"}.`)}>
                              <Power size={14} aria-hidden="true" />
                              {status === "active" ? "Desactivar" : "Activar"}
                            </button>
                          </div>
                        )}
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
