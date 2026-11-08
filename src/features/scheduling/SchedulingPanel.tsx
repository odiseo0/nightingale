import AgendaEditor from "./AgendaEditor";
import AvailabilityPanel from "./AvailabilityPanel";

export default function SchedulingPanel() {
  return (
    <section aria-labelledby="scheduling-heading" className="space-y-4 border-t border-gray-200 pt-6">
      <h2 id="scheduling-heading" className="text-xl font-semibold">
        Agenda y disponibilidad
      </h2>
      <p>
        Los cambios locales quedan provisionales hasta que la API los acepte y se
        descargue su vista. Un provisional, conflicto o rechazo no se presenta como aceptado.
      </p>
      <AgendaEditor />
      <h3 className="text-lg font-semibold">Disponibilidad</h3>
      <AvailabilityPanel />
    </section>
  );
}
