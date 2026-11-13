import { useEffect,useRef,useState } from "react";
import { settingsError } from "./api";
import type { SettingsApi } from "./api";
import type { DataLocation,BackupSummary } from "./types";
import { timestampedBackupFileName } from "./backupFileName";
export function BackupSection({api,onRestored,hasUnsavedChanges=false}:{api:SettingsApi;onRestored:()=>Promise<void>;hasUnsavedChanges?:boolean}) {
  const [location,setLocation]=useState<DataLocation|null>(null);const [busy,setBusy]=useState(false);const [error,setError]=useState("");const [message,setMessage]=useState("");
  const [pending,setPending]=useState<{source:string;summary:BackupSummary}|null>(null);const [confirmed,setConfirmed]=useState(false);
  const dialog=useRef<HTMLDialogElement>(null);const picker=useRef<HTMLButtonElement>(null);
  useEffect(()=>{let active=true;api.getDataLocation().then(v=>{if(active)setLocation(v);}).catch(e=>{if(active)setError(settingsError(e).message);});return()=>{active=false;};},[api]);
  useEffect(()=>{if(pending){dialog.current?.showModal?.();}else{dialog.current?.close?.();}},[pending]);
  async function create(){setBusy(true);setError("");setMessage("");try{const path=await api.chooseBackupDestination(timestampedBackupFileName(new Date()));if(!path)return;const result=await api.createBackup(path);setMessage(`Utworzono kopię: ${result.path}`);}catch(e){setError(settingsError(e).message);}finally{setBusy(false);}}
  async function choose(){setBusy(true);setError("");setMessage("");setPending(null);setConfirmed(false);try{const source=await api.chooseBackupSource();if(!source)return;const summary=await api.inspectBackup(source);setPending({source,summary});}catch(e){setError(settingsError(e).message);}finally{setBusy(false);}}
  function cancel(){setPending(null);setConfirmed(false);picker.current?.focus();}
  async function restore(){if(!pending||!confirmed)return;setBusy(true);setMessage("");setError("");try{const result=await api.restoreBackup(pending.source,true);cancel();
    setMessage(`Przywrócono dane. Kopia poprzedniego stanu: ${result.recovery_backup_path}`);
    try{await onRestored();}catch(e){setError(`Dane przywrócono. Nie udało się odświeżyć panelu: ${settingsError(e).message}`);}
  }catch(e){setError(settingsError(e).message);}finally{setBusy(false);}}
  async function openFolder(){setBusy(true);setError("");try{await api.openDataFolder();}catch(e){setError(settingsError(e).message);}finally{setBusy(false);}}
  return <div><h2>Dane i kopie zapasowe</h2><p className="settings-description">Pełna lokalna baza i zarządzane zasoby. Każdą kopię zapisujesz w wybranym miejscu.</p>
    {location&&<dl className="data-location"><dt>Baza danych</dt><dd>{location.database_path}</dd><dt>Zasoby</dt><dd>{location.assets_dir}</dd></dl>}
    <div className="backup-actions"><button type="button" disabled={busy||!location} onClick={()=>void openFolder()}>Otwórz folder danych</button><button type="button" className="settings-primary" disabled={busy} onClick={()=>void create()}>Utwórz kopię</button><button type="button" ref={picker} disabled={busy} onClick={()=>void choose()}>Wybierz kopię do przywrócenia</button></div>
    <p className="settings-description">Przywrócenie zastępuje całą bazę i logo. Przed zmianą aplikacja tworzy i sprawdza kopię obecnego stanu. Uszkodzone kopie są odrzucane.</p>
    {busy&&<p aria-live="polite">Trwa operacja na danych…</p>}{message&&<p role="status" className="backup-result">{message}</p>}{error&&!pending&&<p role="alert">{error}</p>}
    {pending&&<dialog ref={dialog} aria-modal="true" aria-labelledby="restore-title" className="restore-dialog" onCancel={e=>{e.preventDefault();if(!busy)cancel();}}><h3 id="restore-title">Przywracanie pełnej kopii</h3><p>Sprawdzono integralność kopii:</p><dl className="data-location"><dt>Plik</dt><dd>{pending.source}</dd><dt>Utworzono (UTC)</dt><dd>{pending.summary.created_at_utc}</dd><dt>Wersja aplikacji / schematu</dt><dd>{pending.summary.app_version} / {pending.summary.schema_version}</dd><dt>Pliki / rozmiar</dt><dd>{pending.summary.file_count} / {pending.summary.size_bytes.toLocaleString("pl-PL")} B</dd></dl>
    {hasUnsavedChanges&&<p>Masz niezapisane zmiany w formularzach. Przywrócenie je odrzuci.</p>}
    <label className="restore-consent"><input type="checkbox" checked={confirmed} disabled={busy} onChange={e=>setConfirmed(e.target.checked)}/>Rozumiem, że wszystkie lokalne dane zostaną zastąpione</label>
    {error&&<p role="alert">{error}</p>}
    <div className="backup-actions"><button type="button" disabled={busy} onClick={cancel}>Anuluj przywracanie</button><button type="button" className="settings-primary" disabled={busy||!confirmed} onClick={()=>void restore()}>Zastąp wszystkie lokalne dane</button></div></dialog>}
    </div>;
}
