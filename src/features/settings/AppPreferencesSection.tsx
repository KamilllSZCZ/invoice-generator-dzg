import { useEffect,useState } from "react";
import { settingsError } from "./api";
import type { SettingsApi } from "./api";
import type { AppPreferences } from "./types";
import { SaveFeedback,useSectionForm } from "./SectionForm";
export function AppPreferencesSection({value,api,onSaved,onDirty}:{value:AppPreferences;api:SettingsApi;onSaved:(v:AppPreferences)=>void;onDirty:(b:boolean)=>void}) {
  const f=useSectionForm(value,api.saveAppPreferences,onSaved,onDirty);const [version,setVersion]=useState("");const [error,setError]=useState("");
  useEffect(()=>{let active=true;api.getAppVersion().then(v=>{if(active)setVersion(v);}).catch(e=>{if(active)setError(settingsError(e).message);});return()=>{active=false;};},[api]);
  return <form onSubmit={f.submit}><h2>Ustawienia aplikacji</h2><p className="settings-description">Motyw i gęstość są stosowane po zapisaniu preferencji.</p><fieldset disabled={f.busy}><legend className="sr-only">Preferencje</legend><div className="settings-grid"><div className="settings-field"><label htmlFor="app-theme">Motyw</label><select id="app-theme" value={f.draft.theme??"dark"} onChange={e=>f.update({theme:e.target.value as AppPreferences["theme"]})}><option value="system">Systemowy</option><option value="light">Jasny</option><option value="dark">Ciemny</option></select></div>
    <div className="settings-field"><label htmlFor="app-density">Gęstość interfejsu</label><select id="app-density" value={f.draft.density??"comfortable"} onChange={e=>f.update({density:e.target.value as AppPreferences["density"]})}><option value="comfortable">Komfortowa</option><option value="compact">Kompaktowa</option></select></div></div>
    <p className="settings-version">Wersja aplikacji: <output>{version||"Odczytywanie…"}</output></p>{error&&<p role="alert">{error}</p>}<SaveFeedback {...f} label="Zapisz preferencje"/></fieldset></form>;
}
