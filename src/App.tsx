import { createHashRouter, Navigate, RouterProvider } from "react-router-dom";
import AppLayout from "./app/AppLayout";
import DevelopmentPage from "./app/DevelopmentPage";
import AgendaPage from "./features/appointments/AgendaPage";
import AppointmentPage from "./features/appointments/AppointmentPage";
import AppointmentEditorPage from "./features/appointments/AppointmentEditorPage";
import ReschedulePage from "./features/appointments/ReschedulePage";
import SyncReviewPage from "./features/sync/SyncReviewPage";
import AccessPage from "./features/identity/AccessPage";
import DiagnosticsPage from "./features/diagnostics/DiagnosticsPage";
import SettingsLayout from "./features/settings/SettingsLayout";
import AccountSettingsPage from "./features/settings/AccountSettingsPage";
import ServicesPage from "./features/services/ServicesPage";
import ScheduleSettingsPage from "./features/scheduling/ScheduleSettingsPage";
import PatientsPage from "./features/patients/PatientsPage";
import PatientEditorPage from "./features/patients/PatientEditorPage";
import PatientRecordPage from "./features/patients/PatientRecordPage";
import "./App.css";

// createHashRouter (data router) habilita la API oficial de bloqueo de
// navegación useBlocker para la protección de cambios sin guardar del editor.
// https://reactrouter.com/api/routers/createHashRouter
const router = createHashRouter([
  {
    element: <AppLayout />,
    children: [
      { index: true, element: <Navigate to="/agenda" replace /> },
      { path: "/agenda", element: <AgendaPage /> },
      { path: "/pacientes", element: <PatientsPage /> },
      { path: "/pacientes/nueva", element: <PatientEditorPage /> },
      { path: "/paciente/:patientId/editar", element: <PatientEditorPage /> },
      { path: "/paciente/:patientId", element: <PatientRecordPage /> },
      { path: "/cita/nueva", element: <AppointmentEditorPage /> },
      { path: "/cita/:appointmentId/reprogramar", element: <ReschedulePage /> },
      { path: "/cita/:appointmentId", element: <AppointmentPage /> },
      { path: "/cambios", element: <SyncReviewPage /> },
      { path: "/acceso", element: <AccessPage /> },
      { path: "/diagnostico", element: <DiagnosticsPage /> },
      {
        path: "/configuracion",
        element: <SettingsLayout />,
        children: [
          { index: true, element: <Navigate to="/configuracion/cuenta" replace /> },
          { path: "cuenta", element: <AccountSettingsPage /> },
          { path: "servicios", element: <ServicesPage /> },
          { path: "horarios", element: <ScheduleSettingsPage /> },
        ],
      },
      { path: "/desarrollo", element: <DevelopmentPage /> },
      { path: "*", element: <Navigate to="/agenda" replace /> },
    ],
  },
]);

export default function App() {
  return <RouterProvider router={router} />;
}
