import type { SettingsApi } from "./api";
import type { DocumentDefaults } from "./types";
import { Field,SaveFeedback,useSectionForm } from "./SectionForm";
import { optional,validateDefaults } from "./validation";
export function DocumentDefaultsSection({value,api,onSaved,onDirty}:{value:DocumentDefaults;api:SettingsApi;onSaved:(v:DocumentDefaults)=>void;onDirty:(b:boolean)=>void}) {
  const f=useSectionForm(value,api.saveDocumentDefaults,onSaved,onDirty,validateDefaults);
  return <form onSubmit={f.submit} noValidate><h2>Domyślne ustawienia dokumentów</h2><p className="settings-description">Wartości dla nowych dokumentów. Możesz pozostawić je nieustawione.</p><fieldset disabled={f.busy}><legend className="sr-only">Ustawienia dokumentów</legend><div className="settings-grid">
    <Field name="currency" label="Waluta" value={f.draft.currency??""} onChange={v=>f.update({currency:optional(v.toUpperCase())})} error={f.errors.currency}/>
    <Field name="payment-days" label="Termin płatności (dni)" value={f.draft.payment_term_days?.toString()??""} onChange={v=>f.update({payment_term_days:v===""?null:Number(v)})} error={f.errors.payment_term_days}/>
    <Field name="payment-method" label="Metoda płatności" value={f.draft.payment_method??""} onChange={v=>f.update({payment_method:optional(v)})}/>
    <Field name="issue-place" label="Miejsce wystawienia" value={f.draft.issue_place??""} onChange={v=>f.update({issue_place:optional(v)})}/>
    <Field name="notes" label="Uwagi domyślne" value={f.draft.notes??""} onChange={v=>f.update({notes:optional(v)})} multiline/>
    </div><SaveFeedback {...f} label="Zapisz ustawienia dokumentów"/></fieldset></form>;
}
