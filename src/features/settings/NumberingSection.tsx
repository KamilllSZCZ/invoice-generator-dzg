import { useState } from "react";
import { settingsError } from "./api";
import type { SettingsApi } from "./api";
import type { NumberingSeriesSettings,NumberPreview } from "./types";
import { Field,SaveFeedback,useSectionForm } from "./SectionForm";
import { validateNumbering } from "./validation";
export function NumberingSection({value,api,onSaved,onDirty}:{value:NumberingSeriesSettings;api:SettingsApi;onSaved:(v:NumberingSeriesSettings)=>void;onDirty:(b:boolean)=>void}) {
  const f=useSectionForm(value,api.saveNumberingSettings,onSaved,onDirty,validateNumbering);
  const [preview,setPreview]=useState<NumberPreview|null>(null);const [error,setError]=useState("");const [busy,setBusy]=useState(false);
  async function refresh(){setBusy(true);setError("");try{setPreview(await api.previewNextNumber("invoice"));}catch(e){setError(settingsError(e).message);}finally{setBusy(false);}}
  return <form onSubmit={e=>{setPreview(null);void f.submit(e);}} noValidate><h2>Numeracja faktur</h2><p className="settings-description">Jedna seria dla typu „Faktura”. Zmiany nie modyfikują historycznych numerów.</p><fieldset disabled={f.busy}><legend className="sr-only">Seria faktur</legend><div className="settings-grid">
    <Field name="number-prefix" label="Prefiks" value={f.draft.prefix} onChange={prefix=>f.update({prefix})} error={f.errors.prefix}/>
    <Field name="number-counter" label="Następny licznik" value={String(f.draft.next_counter)} onChange={v=>f.update({next_counter:Number(v)})} error={f.errors.next_counter}/>
    <Field name="number-format" label="Wzór numeru" value={f.draft.format} onChange={format=>f.update({format})} error={f.errors.format}/>
    <div className="settings-field"><label htmlFor="number-reset">Reset licznika</label><select id="number-reset" value={f.draft.reset_policy??"never"} onChange={e=>f.update({reset_policy:e.target.value as NumberingSeriesSettings["reset_policy"]})}><option value="yearly">Co rok</option><option value="monthly">Co miesiąc</option><option value="never">Nigdy</option></select></div>
    </div><p className="settings-description">Dostępne tokeny: <code>{"{prefix} {year} {month} {counter}"}</code>. Zapisz wzór przed podglądem.</p>
    <div className="number-preview"><button type="button" onClick={()=>void refresh()} disabled={busy}>Odśwież podgląd</button>{preview&&<output>{preview.formatted_number}</output>}<p>Podgląd — numer może się zmienić. Dotyczy zapisanej konfiguracji i nie rezerwuje numeru.</p>{error&&<p role="alert">{error}</p>}</div>
    <SaveFeedback {...f} label="Zapisz numerację"/></fieldset></form>;
}
