import BookingPanel from "./BookingPanel";
import AgendaPanel from "./AgendaPanel";

export default function AppointmentsPanel() {
  return (
    <section
      aria-labelledby="appointments-heading"
      className="space-y-4 border-t border-gray-200 pt-6"
    >
      <h2 id="appointments-heading" className="text-xl font-semibold">
        Citas
      </h2>
      <p>
        Las reservas se guardan de inmediato en este dispositivo y quedan
        provisionales hasta que la API las acepte y se descarguen cita e
        historia. Una cita provisional, un conflicto o un rechazo nunca se
        muestran como aceptados, y la hora local jamás se presenta como
        confirmación del servidor.
      </p>
      <BookingPanel />
      <AgendaPanel />
    </section>
  );
}
