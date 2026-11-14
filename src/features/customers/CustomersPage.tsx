import { useEffect, useState } from "react";
import { customersApi, customerError } from "./api";
import type { CustomersApi } from "./api";
import { CustomerForm } from "./CustomerForm";
import { ConfirmDialog } from "../../shared/components/ConfirmDialog";
import { customerInput, emptyCustomer } from "./types";
import type { Customer, CustomerInput } from "./types";
import "./customers.css";

type View = { kind: "none" } | { kind: "create" } | { kind: "detail" | "edit"; customer: Customer };
type Confirmation = { kind: "discard"; next: View } | { kind: "delete"; customer: Customer };
const ignoreDirty = () => {};

function CustomerDetails({ customer, busy, onEdit, onDelete, onClose }: {
  customer: Customer; busy: boolean; onEdit: () => void; onDelete: () => void; onClose: () => void;
}) {
  const address = [customer.address, [customer.postal_code, customer.city].filter(Boolean).join(" "), customer.country].filter(Boolean).join("\n");
  return <div>
    <div className="customer-detail-heading"><h2>{customer.name}</h2><button type="button" onClick={onClose} disabled={busy}>Wróć do listy</button></div>
    <dl className="customer-details">
      {[
        ["NIP / identyfikator podatkowy", customer.tax_id], ["Adres", address],
        ["E-mail", customer.email], ["Telefon", customer.phone], ["Notatki", customer.notes],
      ].map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value || "—"}</dd></div>)}
    </dl>
    <div className="customer-detail-actions"><button type="button" onClick={onEdit} disabled={busy}>Edytuj kontrahenta</button><button type="button" className="customers-danger" onClick={onDelete} disabled={busy}>{busy ? "Usuwanie…" : "Usuń kontrahenta"}</button></div>
  </div>;
}

export function CustomersPage({ api = customersApi, onDirtyChanged = ignoreDirty }: {
  api?: CustomersApi; onDirtyChanged?: (dirty: boolean) => void;
}) {
  const [query, setQuery] = useState("");
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [view, setView] = useState<View>({ kind: "none" });
  const [dirty, setDirty] = useState(false);
  const [mutating, setMutating] = useState(false);
  const [loading, setLoading] = useState(true);
  const [listError, setListError] = useState("");
  const [message, setMessage] = useState("");
  const [revision, setRevision] = useState(0);
  const [confirmation, setConfirmation] = useState<Confirmation | null>(null);
  const [confirmationError, setConfirmationError] = useState("");

  useEffect(() => {
    let active = true;
    setLoading(true);
    setListError("");
    const timer = window.setTimeout(() => {
      api.listCustomers(query).then(result => { if (active) setCustomers(result); })
        .catch(error => { if (active) setListError(customerError(error).message); })
        .finally(() => { if (active) setLoading(false); });
    }, query ? 150 : 0);
    return () => { active = false; window.clearTimeout(timer); };
  }, [api, query, revision]);

  useEffect(() => { onDirtyChanged(dirty); }, [dirty, onDirtyChanged]);
  useEffect(() => {
    if (!dirty) return;
    function beforeUnload(event: BeforeUnloadEvent) { event.preventDefault(); event.returnValue = ""; }
    window.addEventListener("beforeunload", beforeUnload);
    return () => window.removeEventListener("beforeunload", beforeUnload);
  }, [dirty]);

  function applyView(next: View) {
    setDirty(false); setMessage(""); setView(next);
  }

  function changeView(next: View) {
    if (mutating || (next.kind === "create" && view.kind === "create")) return;
    if (dirty) { setConfirmation({ kind: "discard", next }); return; }
    applyView(next);
  }

  async function save(input: CustomerInput) {
    setMutating(true); setMessage("");
    try {
      const saved = view.kind === "edit" ? await api.updateCustomer(view.customer.id, input) : await api.createCustomer(input);
      setDirty(false); setView({ kind: "detail", customer: saved }); setQuery("");
      setRevision(current => current + 1);
      setMessage(view.kind === "edit" ? "Zapisano zmiany kontrahenta." : "Dodano kontrahenta.");
    } finally { setMutating(false); }
  }

  async function remove(customer: Customer) {
    if (mutating) return;
    setMutating(true); setConfirmationError(""); setMessage("");
    try {
      await api.deleteCustomer(customer.id);
      setView({ kind: "none" }); setDirty(false); setRevision(current => current + 1);
      setMessage("Usunięto kontrahenta.");
      setConfirmation(null);
    } catch (error) { setConfirmationError(customerError(error).message); }
    finally { setMutating(false); }
  }

  const editing = view.kind === "create" || view.kind === "edit";
  return <div className={`customers-page${view.kind !== "none" ? " customers-page--selected" : ""}`}>
    <div className="customers-toolbar">
      <div className="customer-field"><label htmlFor="customer-search">Szukaj kontrahenta</label><input id="customer-search" type="search" value={query} maxLength={200} disabled={mutating} onChange={event => setQuery(event.target.value)} aria-describedby="customer-search-help" /></div>
      <button type="button" className={editing ? "" : "customers-primary"} disabled={mutating} onClick={() => changeView({ kind: "create" })}>Dodaj kontrahenta</button>
    </div>
    <p className="customers-description" id="customer-search-help">Nazwa, NIP / identyfikator podatkowy lub miejscowość.</p>
    {message && <p role="status" className="customers-message">{message}</p>}
    <div className="customers-layout">
      <section className="customers-list-panel" aria-labelledby="customer-list-title" aria-busy={loading}>
        <h2 id="customer-list-title">Kartoteka{!loading && !listError ? ` · ${customers.length}` : ""}</h2>
        {loading ? <p role="status" className="customers-description">Wczytywanie kontrahentów…</p> : listError ? <div role="alert"><p>{listError}</p><button type="button" onClick={() => setRevision(current => current + 1)}>Spróbuj ponownie</button></div> : customers.length === 0 ? <p className="customers-empty">{query.trim() ? "Brak kontrahentów pasujących do wyszukiwania." : "Kartoteka jest pusta. Dodaj pierwszego kontrahenta."}</p> :
          <ul className="customer-list">{customers.map(customer => <li key={customer.id}><button type="button" disabled={mutating} aria-current={(view.kind === "detail" || view.kind === "edit") && view.customer.id === customer.id ? "true" : undefined} onClick={() => changeView({ kind: "detail", customer })}>
            <strong>{customer.name}</strong><span>{[customer.tax_id, customer.city].filter(Boolean).join(" · ") || "Brak dodatkowych danych"}</span>
          </button></li>)}</ul>}
      </section>
      <section className="customer-editor" aria-label="Szczegóły kontrahenta">
        {view.kind === "none" ? <div className="customers-empty"><h2>Dane kontrahenta</h2><p>Wybierz kontrahenta z listy lub dodaj nowego.</p></div> : view.kind === "detail" ?
          <CustomerDetails customer={view.customer} busy={mutating} onEdit={() => changeView({ kind: "edit", customer: view.customer })} onDelete={() => { setConfirmationError(""); setConfirmation({ kind: "delete", customer: view.customer }); }} onClose={() => changeView({ kind: "none" })} /> :
          <CustomerForm key={view.kind === "edit" ? view.customer.id : "create"} initial={view.kind === "edit" ? customerInput(view.customer) : emptyCustomer()} editing={view.kind === "edit"} busy={mutating} onSave={save} onCancel={() => changeView(view.kind === "edit" ? { kind: "detail", customer: view.customer } : { kind: "none" })} onDirtyChanged={setDirty} />}
      </section>
    </div>
    {confirmation && <ConfirmDialog title={confirmation.kind === "delete" ? "Usunięcie kontrahenta" : "Niezapisane zmiany kontrahenta"} confirmLabel={confirmation.kind === "delete" ? "Usuń kontrahenta" : "Odrzuć zmiany"} danger={confirmation.kind === "delete"} busy={mutating} onCancel={() => setConfirmation(null)} onConfirm={() => {
      if (confirmation.kind === "delete") void remove(confirmation.customer);
      else { applyView(confirmation.next); setConfirmation(null); }
    }}>
      {confirmation.kind === "delete" ? <><p>Usunąć z kartoteki kontrahenta <strong>{confirmation.customer.name}</strong>?</p><p>Tej operacji nie można cofnąć w kartotece. Kontrahenta powiązanego z dokumentem nie można usunąć.</p></> : <p>Odrzucić niezapisane zmiany kontrahenta?</p>}
      {confirmationError && <p role="alert">{confirmationError}</p>}
    </ConfirmDialog>}
  </div>;
}
