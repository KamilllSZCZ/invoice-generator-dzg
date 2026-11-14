import { useEffect, useId, useRef } from "react";
import type { ReactNode } from "react";
import "./confirm-dialog.css";

export function ConfirmDialog({ title, children, confirmLabel, danger = false, busy = false, onConfirm, onCancel }: {
  title: string; children: ReactNode; confirmLabel: string; danger?: boolean; busy?: boolean;
  onConfirm: () => void; onCancel: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const cancel = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  useEffect(() => {
    const previous = document.activeElement;
    const element = dialog.current;
    if (element && !element.open) {
      if (typeof element.showModal === "function") element.showModal();
      else element.setAttribute("open", "");
    }
    cancel.current?.focus();
    return () => {
      if (element?.open) {
        if (typeof element.close === "function") element.close();
        else element.removeAttribute("open");
      }
      if (previous instanceof HTMLElement && previous.isConnected) previous.focus();
    };
  }, []);
  return <dialog ref={dialog} className="confirmation-dialog" aria-modal="true" aria-labelledby={titleId} onCancel={event => { event.preventDefault(); if (!busy) onCancel(); }}>
    <h2 id={titleId}>{title}</h2>
    <div className="confirmation-copy">{children}</div>
    <div className="confirmation-actions">
      <button type="button" ref={cancel} disabled={busy} onClick={onCancel}>Anuluj</button>
      <button type="button" className={danger ? "confirmation-danger" : "confirmation-primary"} disabled={busy} onClick={onConfirm}>{busy ? "Trwa operacja…" : confirmLabel}</button>
    </div>
  </dialog>;
}
