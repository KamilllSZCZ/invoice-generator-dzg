import { useState } from "react";
import type { SettingsApi } from "./api";
import { settingsError } from "./api";
import type { SellerProfile } from "./types";
import { Field,SaveFeedback,useSectionForm } from "./SectionForm";
import { optional,validateCompany } from "./validation";
export function CompanySection({value,api,onSaved,onDirty,logo,onLogoChanged}:{value:SellerProfile;api:SettingsApi;onSaved:(v:SellerProfile)=>void;onDirty:(b:boolean)=>void;logo:string|null;onLogoChanged:()=>Promise<void>}) {
  const f=useSectionForm(value,api.saveCompanySettings,onSaved,onDirty,validateCompany);
  const [logoBusy,setLogoBusy]=useState(false);const [logoError,setLogoError]=useState("");
  async function changeLogo(remove=false){setLogoBusy(true);setLogoError("");try {
    if(remove)await api.removeCompanyLogo();else {const path=await api.chooseLogo();if(!path)return;await api.setCompanyLogo(path);}
    await onLogoChanged();
  }catch(e){setLogoError(settingsError(e).message);}finally{setLogoBusy(false);}}
  const fields:[keyof SellerProfile,string][]=[["name","Nazwa sprzedawcy"],["address","Ulica i numer"],["postal_code","Kod pocztowy"],["city","Miejscowość"],["country","Kraj"],["email","E-mail"],["phone","Telefon"],["website","Strona internetowa"],["bank_account","Rachunek bankowy"]];
  return <form onSubmit={f.submit} noValidate><h2>Dane sprzedawcy</h2><p className="settings-description">Uzupełnij wyłącznie dane, których używasz. Wszystkie pola są opcjonalne.</p>
    <fieldset disabled={f.busy||logoBusy}><legend className="sr-only">Profil sprzedawcy</legend><div className="settings-grid">{fields.map(([key,label])=><Field key={key} name={`company-${key}`} label={label} value={f.draft[key]??""} onChange={v=>f.update({[key]:optional(v)})} error={f.errors[key]}/>)}</div>
    <div className="logo-tools"><div className="logo-preview">{logo?<img src={logo} alt="Logo sprzedawcy"/>:<span>Brak logo</span>}</div><div><button type="button" onClick={()=>void changeLogo()}>Wybierz logo</button> <button type="button" disabled={!logo} onClick={()=>void changeLogo(true)}>Usuń logo</button><p className="settings-description">PNG lub JPEG, do 10 MiB i 25 mln pikseli. Zmiana logo jest zapisywana od razu.</p></div></div>
    {logoError&&<p role="alert">{logoError}</p>}<SaveFeedback {...f} label="Zapisz dane sprzedawcy"/></fieldset></form>;
}
