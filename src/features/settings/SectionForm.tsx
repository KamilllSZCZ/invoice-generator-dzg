import { useEffect,useState } from "react";
import type { ReactNode,FormEvent } from "react";
import { settingsError } from "./api";
import type { FieldErrors } from "./types";

export function useSectionForm<T>(value:T,save:(input:T)=>Promise<T>,onSaved:(value:T)=>void,onDirty:(dirty:boolean)=>void,validate:(value:T)=>FieldErrors=()=>({})) {
  const [draft,setDraft]=useState(value);const [busy,setBusy]=useState(false);const [message,setMessage]=useState("");const [errors,setErrors]=useState<FieldErrors>({});
  const dirty=JSON.stringify(draft)!==JSON.stringify(value);
  useEffect(()=>{setDraft(value);},[value]);
  useEffect(()=>{onDirty(dirty);},[dirty,onDirty]);
  function update(patch:Partial<T>){setDraft(current=>({...current,...patch}));setMessage("");setErrors({});}
  async function submit(event:FormEvent){event.preventDefault();const fields=validate(draft);setErrors(fields);setMessage("");if(Object.keys(fields).length)return;
    setBusy(true);try{const saved=await save(draft);setDraft(saved);onSaved(saved);setMessage("Zapisano zmiany.");}catch(error){const e=settingsError(error);setErrors(e.field_errors??{_form:e.message});}finally{setBusy(false);}}
  return {draft,busy,message,errors,dirty,update,submit};
}
export function SaveFeedback({busy,dirty,message,errors,label}:{busy:boolean;dirty:boolean;message:string;errors:FieldErrors;label:string}) {
  return <><div aria-live="polite" className="save-feedback">{Object.keys(errors).length>0&&<div role="alert">{Object.values(errors).map((e,i)=><p key={i}>{e}</p>)}</div>}{message&&<p role="status">{message}</p>}{dirty&&<p>Niezapisane zmiany</p>}</div><button className="settings-primary" type="submit" disabled={busy||!dirty}>{busy?"Zapisywanie…":label}</button></>;
}
export function Field({name,label,value,onChange,error,multiline=false,type="text",children}:{name:string;label:string;value:string;onChange:(value:string)=>void;error?:string;multiline?:boolean;type?:string;children?:ReactNode}) {
  const props={id:name,name,value,onChange:(e:React.ChangeEvent<HTMLInputElement|HTMLTextAreaElement>)=>onChange(e.target.value),"aria-invalid":!!error,"aria-describedby":error?`${name}-error`:undefined};
  return <div className={`settings-field${multiline?" settings-field--wide":""}`}><label htmlFor={name}>{label}</label>{multiline?<textarea {...props} rows={3}/>:<input {...props} type={type}/ >}{children}{error&&<small id={`${name}-error`}>{error}</small>}</div>;
}
