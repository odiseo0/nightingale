import { useState } from "react";
import { getDiagnostics, type Diagnostics } from "./diagnostics";
import "./App.css";

type DiagnosticState =
  | { status: "idle" | "loading" }
  | { status: "ready"; data: Diagnostics }
  | { status: "error" };

function App() {
  const [diagnostic, setDiagnostic] = useState<DiagnosticState>({ status: "idle" });

  async function loadDiagnostics() {
    setDiagnostic({ status: "loading" });

    try {
      const data = await getDiagnostics();
      setDiagnostic({ status: "ready", data });
    } catch {
      setDiagnostic({ status: "error" });
    }
  }

  return (
    <main className="mx-auto max-w-3xl space-y-8 px-6 py-10">
      <header className="space-y-2">
        <h1>Nightingale</h1>
        <p>Base de desarrollo del escritorio.</p>
      </header>

      <section aria-labelledby="diagnostics-heading" className="space-y-4">
        <h2 id="diagnostics-heading" className="text-xl font-semibold">
          Diagnóstico local
        </h2>
        <p>Comprueba la comunicación entre esta interfaz y el proceso Rust.</p>
        <button
          type="button"
          className="disabled:cursor-wait disabled:opacity-60"
          onClick={loadDiagnostics}
          disabled={diagnostic.status === "loading"}
        >
          {diagnostic.status === "loading" ? "Consultando…" : "Consultar diagnóstico"}
        </button>
        <p role="status">
          {diagnostic.status === "idle" && "Sin consultar."}
          {diagnostic.status === "loading" && "Consultando el proceso local…"}
          {diagnostic.status === "ready" && "Diagnóstico recibido desde Rust."}
        </p>

        {diagnostic.status === "error" && (
          <p role="alert">
            No se pudo consultar el diagnóstico local. Comprueba que usas la
            aplicación de escritorio y vuelve a intentarlo.
          </p>
        )}

        {diagnostic.status === "ready" && (
          <dl className="grid gap-4 sm:grid-cols-3">
            <div>
              <dt className="text-sm text-gray-600">Versión de la app</dt>
              <dd className="font-mono">{diagnostic.data.appVersion}</dd>
            </div>
            <div>
              <dt className="text-sm text-gray-600">Sistema operativo</dt>
              <dd className="font-mono">{diagnostic.data.platform}</dd>
            </div>
            <div>
              <dt className="text-sm text-gray-600">Arquitectura</dt>
              <dd className="font-mono">{diagnostic.data.architecture}</dd>
            </div>
          </dl>
        )}
      </section>

      <section aria-labelledby="scope-heading" className="space-y-2 border-t border-gray-200 pt-6">
        <h2 id="scope-heading" className="text-xl font-semibold">
          Funciones pendientes
        </h2>
        <p>
          La gestión de pacientes, citas, sesiones y sincronización aún no está
          implementada. Los servicios web tendrán sus propios planes.
        </p>
      </section>
    </main>
  );
}

export default App;
