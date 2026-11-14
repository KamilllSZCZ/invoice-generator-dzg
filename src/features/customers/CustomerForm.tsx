import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { customerError } from "./api";
import type { CustomerInput, CustomerFieldErrors } from "./types";

const fields: { key: keyof CustomerInput; label: string; limit: number; type?: string }[] = [
  { key: "name", label: "Nazwa kontrahenta", limit: 200 },
  { key: "tax_id", label: "NIP / identyfikator podatkowy", limit: 40 },
  { key: "address", label: "Ulica i numer", limit: 300 },
  { key: "postal_code", label: "Kod pocztowy", limit: 30 },
  { key: "city", label: "Miejscowość", limit: 100 },
  { key: "country", label: "Kraj", limit: 100 },
  { key: "email", label: "E-mail", limit: 254, type: "email" },
  { key: "phone", label: "Telefon", limit: 50, type: "tel" },
  { key: "notes", label: "Notatki", limit: 2000 },
];

export function CustomerForm({ initial, editing, busy, onSave, onCancel, onDirtyChanged }: {
  initial: CustomerInput;
  editing: boolean;
  busy: boolean;
  onSave: (input: CustomerInput) => Promise<void>;
  onCancel: () => void;
  onDirtyChanged: (dirty: boolean) => void;
}) {
  const [draft, setDraft] = useState(initial);
  const [errors, setErrors] = useState<CustomerFieldErrors>({});
  const dirty = JSON.stringify(draft) !== JSON.stringify(initial);
  useEffect(() => { onDirtyChanged(dirty); }, [dirty, onDirtyChanged]);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (busy) return;
    setErrors({});
    if (!draft.name.trim()) {
      setErrors({ name: "Podaj nazwę kontrahenta." });
      return;
    }
    try { await onSave(draft); }
    catch (error) {
      const result = customerError(error);
      setErrors({ ...result.field_errors, _form: result.message });
    }
  }

  return <form onSubmit={event => void submit(event)} noValidate aria-labelledby="customer-form-title">
    <h2 id="customer-form-title">{editing ? "Edycja kontrahenta" : "Nowy kontrahent"}</h2>
    <p className="customers-description">Wymagana jest tylko nazwa. Pozostałe dane możesz uzupełnić później.</p>
    <fieldset disabled={busy} className="customer-fields">
      <legend className="customers-sr-only">Dane kontrahenta</legend>
      {fields.map(field => {
        const id = `customer-${field.key}`;
        const props = {
          id, name: field.key, value: draft[field.key] ?? "", maxLength: field.limit,
          onChange: (event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
            const value = event.target.value;
            setDraft(current => ({ ...current, [field.key]: field.key === "name" ? value : value || null }));
            setErrors({});
          },
          "aria-invalid": !!errors[field.key],
          "aria-describedby": errors[field.key] ? `${id}-error` : undefined,
        };
        return <div className={`customer-field${field.key === "name" || field.key === "notes" ? " customer-field--wide" : ""}`} key={field.key}>
          <label htmlFor={id}>{field.label}{field.key === "name" ? " (wymagane)" : ""}</label>
          {field.key === "notes" ? <textarea {...props} rows={3} /> : <input {...props} type={field.type ?? "text"} required={field.key === "name"} />}
          {errors[field.key] && <small id={`${id}-error`}>{errors[field.key]}</small>}
        </div>;
      })}
    </fieldset>
    {errors._form && <p role="alert">{errors._form}</p>}
    {Object.keys(errors).some(key => key !== "_form") && <p role="alert">Popraw zaznaczone pola.</p>}
    <div className="customer-form-actions">
      <button type="button" disabled={busy} onClick={onCancel}>Anuluj</button>
      <button type="submit" className="customers-primary" disabled={busy || (editing && !dirty)}>{busy ? "Zapisywanie…" : "Zapisz kontrahenta"}</button>
      {dirty && <span className="customers-description">Niezapisane zmiany</span>}
    </div>
  </form>;
}
