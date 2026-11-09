import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { ArrowLeft } from "@phosphor-icons/react";
import { getBookingContext, type BookingContext } from "../../shared/api/appointments";
import AppointmentDetail from "./AppointmentDetail";

/** Superficie propia del detalle (S03) para ventanas estrechas o acceso
 * directo a /cita/:appointmentId. Conserva fecha y filtros de la agenda. */
export default function AppointmentPage() {
  const { appointmentId } = useParams<{ appointmentId: string }>();
  const navigate = useNavigate();
  const [context, setContext] = useState<BookingContext | null>(null);

  useEffect(() => {
    let cancelled = false;
    getBookingContext()
      .then((data) => { if (!cancelled) setContext(data); })
      .catch(() => { /* contexto no disponible: el detalle usa la zona de la cita */ });
    return () => { cancelled = true; };
  }, []);

  if (!appointmentId) {
    return (
      <div className="state-block" role="alert">
        <p>El identificador de cita no es válido.</p>
        <button type="button" className="btn" onClick={() => navigate("/agenda")}>
          Volver a la agenda
        </button>
      </div>
    );
  }

  return (
    <>
      <div>
        <button
          type="button"
          className="btn btn--subtle"
          onClick={() => navigate("/agenda")}
        >
          <ArrowLeft size={16} aria-hidden="true" />
          Volver a la agenda
        </button>
      </div>
      <div style={{ width: "100%" }}>
        <AppointmentDetail appointmentId={appointmentId} context={context} />
      </div>
    </>
  );
}
