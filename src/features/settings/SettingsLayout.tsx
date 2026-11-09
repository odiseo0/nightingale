import { NavLink, Outlet } from "react-router-dom";

/** S12 · Configuración con subnavegación. Solo destinos funcionales:
 *  las secciones futuras (servicios, horario) se añadirán con sus planes. */
export default function SettingsLayout() {
  return (
    <>
      <nav className="tabs" aria-label="Secciones de configuración" style={{ marginBottom: 4 }}>
        <NavLink to="/configuracion/cuenta" className="tab">
          Cuenta
        </NavLink>
        <NavLink to="/configuracion/servicios" className="tab">
          Servicios
        </NavLink>
        <NavLink to="/configuracion/horarios" className="tab">
          Horarios
        </NavLink>
      </nav>
      <Outlet />
    </>
  );
}
