import type { SettingsApi } from "./api";
import type { AppSettings,BackupSummary } from "./types";
export function initialSettings():AppSettings {return {
  company:{name:null,address:null,postal_code:null,city:null,country:null,tax_id:null,registration_id:null,email:null,phone:null,website:null,bank_account:null},
  document_defaults:{document_type:"invoice",currency:null,payment_method:null,payment_term_days:null,issue_place:null,notes:null},
  numbering:[{document_type:"invoice",prefix:"",format:"{counter}/{year}",reset_policy:"yearly",next_counter:1}],
  appearance:{template:null,accent_color:null,show_logo:null,footer:null,show_address:null,show_email:null,show_phone:null,show_website:null,show_bank_account:null},
  preferences:{language:null,theme:null,density:null},
};}
export const backup:BackupSummary={path:"C:/kopie/kopia-01.zip",format_version:1,schema_version:2,app_version:"0.1.0",created_at_utc:"2026-11-13T18:00:00Z",file_count:2,size_bytes:40960,fingerprint:"a".repeat(64)};
export function fakeApi():SettingsApi&{saved:()=>AppSettings} {
  let data=initialSettings();return {
    saved:()=>structuredClone(data),getSettings:async()=>structuredClone(data),getAppVersion:async()=>"0.1.0",getCompanyLogoDataUrl:async()=>null,
    saveCompanySettings:async input=>{data.company=structuredClone(input);return input;},setCompanyLogo:async()=>data.company,removeCompanyLogo:async()=>data.company,
    saveDocumentDefaults:async input=>{data.document_defaults=structuredClone(input);return input;},getNumberingSettings:async()=>structuredClone(data.numbering),
    saveNumberingSettings:async input=>{data.numbering=[structuredClone(input)];return input;},previewNextNumber:async()=>({document_type:"invoice",formatted_number:"1/2026",period_key:"2026",counter:data.numbering[0].next_counter}),
    saveAppearanceSettings:async input=>{data.appearance=structuredClone(input);return input;},saveAppPreferences:async input=>{data.preferences=structuredClone(input);return input;},
    createBackup:async()=>backup,inspectBackup:async()=>backup,restoreBackup:async()=>({restored_schema_version:2,recovery_backup_path:"C:/recovery/before-restore.zip"}),
    getDataLocation:async()=>({app_data_dir:"C:/app",database_path:"C:/app/data/app.sqlite",assets_dir:"C:/app/data/assets"}),openDataFolder:async()=>{},chooseLogo:async()=>null,chooseBackupDestination:async()=>null,chooseBackupSource:async()=>null,
  };
}
