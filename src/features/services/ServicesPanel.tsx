import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  listCatalog, createService, updateService, setServiceStatus, offerService,
  updateServiceOffer, setServiceOfferStatus, updateProfessionalProfile, getCatalogPreferences, syncCatalog,
  type CatalogScreen, type LocalChange, type ServicePatch,
} from "../../shared/api/services";

const field = (data: FormData, name: string) => String(data.get(name) ?? "");
const adjustment = (data: FormData) => field(data, "adjustment") === "" ? null : Number(field(data, "adjustment"));
const pending = (changes: LocalChange[]) => changes.some(change => !["conflict", "rejected"].includes(change.syncStatus));
const labels: Record<LocalChange["syncStatus"], string> = {
  pending: "Pendiente", in_flight: "Enviando", unknown: "Resultado desconocido", retryable: "Esperando reintento",
  accepted: "Aceptada; falta confirmar la vista", conflict: "Conflicto", rejected: "Rechazada", review_required: "Requiere revisión",
};

function Changes({ changes }: { changes: LocalChange[] }) {
  if (!changes.length) return null;
  return <ul aria-label="Cambios locales" className="space-y-1 text-sm">{changes.map(change => <li key={change.operationId} className="break-all">
    {labels[change.syncStatus]}{change.code ? ` · ${change.code}` : ""} · <code>{change.operationId}</code>
    {["conflict", "rejected", "review_required"].includes(change.syncStatus) && <> · <a href="#core-heading">Ver revisión de sincronización</a></>}
  </li>)}</ul>;
}

export default function ServicesPanel() {
  const [screen, setScreen] = useState<CatalogScreen | null>(null);
  const [showInactive, setShowInactive] = useState(true);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const generation = useRef(0);

  // Local IPC reads share the account panel's cleanup/late-response pattern.
  // https://react.dev/reference/react/useEffect#fetching-data-with-effects
  useEffect(() => {
    const refresh = async () => {
      const request = ++generation.current;
      try {
        const [view, preferences] = await Promise.all([listCatalog(), getCatalogPreferences()]);
        if (request === generation.current) { setScreen(view); setShowInactive(preferences.servicesShowInactive); }
      } catch { if (request === generation.current) setScreen(null); }
    };
    void refresh();
    const timer = window.setInterval(() => void refresh(), 2000);
    return () => { ++generation.current; window.clearInterval(timer); };
  }, []);

  async function act(work: () => Promise<unknown>, success = "Cambio guardado en este dispositivo; pendiente de aceptación.") {
    setBusy(true); setMessage("");
    try {
      await work(); setMessage(success);
      const request = ++generation.current;
      const view = await listCatalog();
      if (request === generation.current) setScreen(view);
    } catch (error: unknown) {
      const code = typeof error === "object" && error !== null && "code" in error ? String(error.code) : "TEMPORARY_FAILURE";
      setMessage(`No se pudo completar la acción (${code}). Revisa el acceso, las versiones y los cambios pendientes.`);
    } finally { setBusy(false); }
  }

  function create(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); const data = new FormData(event.currentTarget);
    void act(() => createService({ professionalId: field(data, "professional"), name: field(data, "name"), defaultDurationMinutes: Number(field(data, "duration")) }));
  }

  return <section aria-labelledby="services-heading" className="space-y-4 border-t border-gray-200 pt-6">
    <h2 id="services-heading" className="text-xl font-semibold">Catálogo de servicios</h2>
    {!screen && <p>Inicia sesión y elige una cuenta con acceso activo para consultar el catálogo local.</p>}
    {screen && <>
      <p>Los cambios locales quedan provisionales hasta que la API los acepte y se descargue su vista.</p>
      {!screen.canManage && <p>Catálogo de solo lectura para este acceso.</p>}
      <button disabled={busy} onClick={() => void act(syncCatalog, "Se solicitó la sincronización. Revisa el estado de cada cambio.")}>Sincronizar catálogo</button>
      {screen.canManage && <form onSubmit={create}>
        <fieldset disabled={busy || !screen.professionals.some(row => row.accepted)} className="grid gap-3 sm:grid-cols-2">
          <legend>Nuevo servicio</legend>
          <label className="grid gap-1">Profesional<select name="professional" required className="rounded border p-2">{screen.professionals.map(row => row.accepted && <option key={row.accepted.professionalId} value={row.accepted.professionalId}>{row.accepted.publicName}</option>)}</select></label>
          <label className="grid gap-1">Nombre<input name="name" required /></label>
          <label className="grid gap-1">Duración base (minutos)<input name="duration" type="number" min="1" max="1440" step="1" required defaultValue="30" /></label>
          <button type="submit">Crear servicio local</button>
        </fieldset>
        {!screen.professionals.some(row => row.accepted) && <p>Sincroniza para descargar el perfil profesional antes de crear servicios.</p>}
      </form>}

      <h3 className="font-semibold">Servicios{!showInactive && " activos"}</h3>
      {!screen.services.length && <p>No hay servicios en la vista local.</p>}
      <ul className="space-y-5">{screen.services.map(row => {
        const service = row.provisional ?? row.accepted;
        if (!service || (!showInactive && service.status === "inactive")) return null;
        const accepted = row.accepted;
        const blocked = busy || pending(row.localChanges);
        const hasOffer = screen.offers.some(offer => (offer.provisional ?? offer.accepted)?.serviceId === service.serviceId && (offer.provisional ?? offer.accepted)?.professionalId === service.professionalId);
        return <li key={service.serviceId} className="space-y-3 rounded border p-4">
          <h4 className="font-semibold">{service.name} · {service.status === "active" ? "Activo" : "Inactivo"}{row.isProvisional && " · Propuesta local"}</h4>
          {accepted ? <p>Aceptado: {accepted.name}, {accepted.defaultDurationMinutes} minutos · versión {accepted.version}.</p> : <p>Sin versión aceptada.</p>}
          {row.provisional && <p>Propuesta: {row.provisional.defaultDurationMinutes} minutos.</p>}
          <Changes changes={row.localChanges} />
          {screen.canManage && accepted && <>
            <form key={`edit-${service.serviceId}-${accepted.version}`} onSubmit={event => {
              event.preventDefault(); const data = new FormData(event.currentTarget); const patch: ServicePatch = {};
              if (field(data, "name") !== accepted.name) patch.name = field(data, "name");
              if (Number(field(data, "duration")) !== accepted.defaultDurationMinutes) patch.defaultDurationMinutes = Number(field(data, "duration"));
              void act(() => updateService({ serviceId: service.serviceId, expectedVersion: accepted.version, patch }));
            }}>
              <fieldset disabled={blocked} className="grid gap-3 sm:grid-cols-2">
                <legend>Editar servicio</legend>
                <label className="grid gap-1">Nombre<input name="name" required defaultValue={service.name} /></label>
                <label className="grid gap-1">Duración base<input name="duration" type="number" min="1" max="1440" step="1" required defaultValue={service.defaultDurationMinutes} /></label>
                <button type="submit">Guardar cambio local</button>
              </fieldset>
            </form>
            <button disabled={blocked} onClick={() => void act(() => setServiceStatus({ serviceId: service.serviceId, expectedVersion: accepted.version, status: accepted.status === "active" ? "inactive" : "active" }))}>{accepted.status === "active" ? "Desactivar servicio" : "Activar servicio"}</button>
          </>}
          {screen.canManage && !hasOffer && <form onSubmit={event => {
            event.preventDefault(); const data = new FormData(event.currentTarget);
            void act(() => offerService({ professionalId: service.professionalId, serviceId: service.serviceId, durationMinutesOverride: adjustment(data) }));
          }}>
            <fieldset disabled={busy} className="space-y-2"><legend>Ofrecer en esta cuenta</legend>
              <label className="grid gap-1">Duración de oferta; vacío usa la base<input name="adjustment" type="number" min="1" max="1440" step="1" /></label>
              <button type="submit">Guardar oferta local</button>
            </fieldset>
          </form>}
        </li>;
      })}</ul>

      <h3 className="font-semibold">Ofertas de la cuenta</h3>
      {!screen.offers.length && <p>No hay ofertas en la vista local.</p>}
      <ul className="space-y-4">{screen.offers.map(row => {
        const offer = row.provisional ?? row.accepted;
        if (!offer || (!showInactive && offer.status === "inactive")) return null;
        const accepted = row.accepted;
        const blocked = busy || pending(row.localChanges);
        const service = screen.services.find(service => (service.provisional ?? service.accepted)?.serviceId === offer.serviceId);
        const name = accepted?.name ?? service?.provisional?.name ?? service?.accepted?.name ?? offer.serviceId;
        return <li key={`${offer.professionalId}-${offer.serviceId}`} className="space-y-3 rounded border p-4">
          <h4 className="font-semibold">{name}{row.isProvisional && " · Propuesta local"}</h4>
          {accepted ? <p>Duración efectiva aceptada: {accepted.effectiveDurationMinutes} minutos · oferta v{accepted.version}, servicio v{accepted.serviceVersion} · {accepted.status === "active" ? "Activa" : "Inactiva"}.</p> : <p>Sin duración efectiva aceptada.</p>}
          {row.provisional && <p>Ajuste propuesto: {row.provisional.durationMinutesOverride == null ? "usar duración base" : `${row.provisional.durationMinutesOverride} minutos`}.</p>}
          <Changes changes={row.localChanges} />
          {screen.canManage && <form key={`offer-${offer.professionalId}-${offer.serviceId}-${accepted?.version ?? "new"}`} onSubmit={event => {
            event.preventDefault(); const data = new FormData(event.currentTarget);
            const request = { professionalId: offer.professionalId, serviceId: offer.serviceId, durationMinutesOverride: adjustment(data) };
            void act(() => accepted ? updateServiceOffer({ ...request, expectedVersion: accepted.version }) : offerService(request));
          }}>
            <fieldset disabled={blocked} className="space-y-2"><legend>{accepted ? "Ajustar duración" : "Revisar y volver a ofrecer"}</legend>
              <label className="grid gap-1">Minutos; vacío vuelve a la base<input name="adjustment" type="number" min="1" max="1440" step="1" defaultValue={offer.durationMinutesOverride ?? ""} /></label>
              <button type="submit">Guardar oferta local</button>
            </fieldset>
          </form>}
          {screen.canManage && accepted && <button disabled={blocked} onClick={() => void act(() => setServiceOfferStatus({ professionalId: offer.professionalId, serviceId: offer.serviceId, expectedVersion: accepted.version, status: accepted.status === "active" ? "inactive" : "active" }))}>{accepted.status === "active" ? "Desactivar oferta" : "Activar oferta"}</button>}
        </li>;
      })}</ul>

      <h3 className="font-semibold">Perfiles profesionales</h3>
      {screen.professionals.map(row => {
        const profile = row.provisional ?? row.accepted;
        if (!profile) return null;
        const accepted = row.accepted;
        return <div key={profile.professionalId} className="space-y-2 rounded border p-4">
          <p>{profile.publicName}{row.isProvisional && " · Propuesta local"}</p>
          <Changes changes={row.localChanges} />
          {screen.canManage && accepted && <form key={`${profile.professionalId}-${accepted.version}`} onSubmit={event => {
            event.preventDefault(); const data = new FormData(event.currentTarget);
            void act(() => updateProfessionalProfile({ professionalId: profile.professionalId, expectedVersion: accepted.version, publicName: field(data, "publicName") }));
          }}>
            <fieldset disabled={busy || pending(row.localChanges)} className="space-y-2"><legend>Nombre público</legend>
              <label className="grid gap-1">Nombre<input name="publicName" required defaultValue={profile.publicName} /></label><button type="submit">Guardar nombre local</button>
            </fieldset>
          </form>}
        </div>;
      })}
    </>}
    {message && <p role="status">{message}</p>}
  </section>;
}
