import { useEffect, useRef, type ReactNode } from "react";

interface Props {
  open: boolean;
  onClose: () => void;
  /** Nombre accesible cuando el diálogo no tenga un encabezado propio. */
  label?: string;
  /** Id del encabezado que etiqueta el diálogo (preferido sobre `label`). */
  labelledBy?: string;
  /** Usar alertdialog para confirmaciones que exigen una decisión. */
  alert?: boolean;
  maxWidth?: number;
  children: ReactNode;
}

/** Diálogo modal común. <dialog> con showModal() aporta la condición modal,
 * la inactividad del fondo, Escape y el retorno de foco al disparador.
 * Source: https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dialog */
export default function Modal({ open, onClose, label, labelledBy, alert = false, maxWidth = 440, children }: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  }, [open]);

  return (
    <dialog
      ref={dialogRef}
      className="modal"
      role={alert ? "alertdialog" : "dialog"}
      aria-modal="true"
      aria-label={label}
      aria-labelledby={labelledBy}
      style={{ maxWidth }}
      onClose={onClose}
      onClick={(event) => {
        // Clic sobre el propio <dialog> (el scrim) cierra; el contenido no propaga.
        if (event.target === dialogRef.current) dialogRef.current.close();
      }}
    >
      {children}
    </dialog>
  );
}
