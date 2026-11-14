import { useEffect,useState } from "react";
import { settingsApi,settingsError } from "./api";
import type { SettingsApi } from "./api";
import type { AppSettings,AppPreferences } from "./types";
import { CompanySection } from "./CompanySection";
import { DocumentDefaultsSection } from "./DocumentDefaultsSection";
import { NumberingSection } from "./NumberingSection";
import { AppearanceSection } from "./AppearanceSection";
import { AppPreferencesSection } from "./AppPreferencesSection";
import { BackupSection } from "./BackupSection";
import "./settings.css";
export function SettingsPage({api=settingsApi,onPreferencesChanged,onDirtyChanged}:{api?:SettingsApi;onPreferencesChanged?:(p:AppPreferences)=>void;onDirtyChanged?:(b:boolean)=>void}) {
  const [settings,setSettings]=useState<AppSettings|null>(null);const [error,setError]=useState("");const [loading,setLoading]=useState(true);const [active,setActive]=useState("company");
  const [logo,setLogo]=useState<string|null>(null);const [dirty,setDirty]=useState<Record<string,boolean>>({});
  async function load(){setLoading(true);setError("");try{const [s,n,l]=await Promise.all([api.getSettings(),api.getNumberingSettings(),api.getCompanyLogoDataUrl()]);setSettings({...s,numbering:n});setLogo(l);setDirty({});onPreferencesChanged?.(s.preferences);}catch(e){setError(settingsError(e).message);}finally{setLoading(false);}}
  useEffect(()=>{void load();},[api]);
  const anyDirty=Object.values(dirty).some(Boolean);
  useEffect(()=>{onDirtyChanged?.(anyDirty);const warn=(e:BeforeUnloadEvent)=>{if(anyDirty){e.preventDefault();e.returnValue="";}};window.addEventListener("beforeunload",warn);return()=>window.removeEventListener("beforeunload",warn);},[anyDirty,onDirtyChanged]);
  function markDirty(key:string,b:boolean){setDirty(previous=>previous[key]===b?previous:{...previous,[key]:b});}
  function saved<K extends keyof AppSettings>(key:K,value:AppSettings[K]){setSettings(previous=>previous?{...previous,[key]:value}:previous);}
  if(loading)return <p role="status">Wczytywanie ustawień…</p>;
  if(error||!settings)return <div role="alert"><p>{error||"Nie można odczytać ustawień."}</p><button onClick={()=>void load()}>Spróbuj ponownie</button></div>;
  async function refreshAfterRestore(){const [s,n,l]=await Promise.all([api.getSettings(),api.getNumberingSettings(),api.getCompanyLogoDataUrl()]);setSettings({...s,numbering:n});setLogo(l);setDirty({});onPreferencesChanged?.(s.preferences);}
  return <div className="settings-layout"><nav className="settings-nav" aria-label="Sekcje ustawień">{[["company","Dane sprzedawcy"],["documents","Dokumenty"],["numbering","Numeracja"],["appearance","Wygląd dokumentów"],["preferences","Aplikacja"],["backup","Kopie zapasowe"]].map(([key,label])=><button type="button" aria-label={label} aria-current={key===active?"page":undefined} key={key} onClick={()=>setActive(key)}>{label}{dirty[key]&&<span aria-hidden="true"> ●</span>}</button>)}</nav>
    <div className="settings-body">
    <section hidden={active!=="company"}><CompanySection value={settings.company} api={api} onSaved={v=>saved("company",v)} onDirty={b=>markDirty("company",b)} logo={logo} onLogoChanged={async()=>setLogo(await api.getCompanyLogoDataUrl())}/></section>
    <section hidden={active!=="documents"}><DocumentDefaultsSection value={settings.document_defaults} api={api} onSaved={v=>saved("document_defaults",v)} onDirty={b=>markDirty("documents",b)}/></section>
    <section hidden={active!=="numbering"}><NumberingSection value={settings.numbering[0]} api={api} onSaved={v=>saved("numbering",[v])} onDirty={b=>markDirty("numbering",b)}/></section>
    <section hidden={active!=="appearance"}><AppearanceSection value={settings.appearance} api={api} onSaved={v=>saved("appearance",v)} onDirty={b=>markDirty("appearance",b)} logo={logo}/></section>
    <section hidden={active!=="preferences"}><AppPreferencesSection value={settings.preferences} api={api} onSaved={v=>{saved("preferences",v);onPreferencesChanged?.(v);}} onDirty={b=>markDirty("preferences",b)}/></section>
    <section hidden={active!=="backup"}><BackupSection api={api} hasUnsavedChanges={anyDirty} onRestored={refreshAfterRestore}/></section>
    </div></div>;
}
