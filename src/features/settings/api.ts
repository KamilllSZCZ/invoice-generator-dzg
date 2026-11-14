import { invoke } from "@tauri-apps/api/core";
import { open,save } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import type { AppSettings,SellerProfile,DocumentDefaults,NumberingSeriesSettings,DocumentAppearanceSettings,AppPreferences,NumberPreview,BackupSummary,RestoreSummary,DataLocation,SettingsError } from "./types";
export function settingsError(error:unknown):SettingsError {
  if(typeof error==="object"&&error!==null&&"message" in error) {
    const e=error as SettingsError;return {code:e.code??"error",message:String(e.message),field_errors:e.field_errors};
  }
  return {code:"error",message:typeof error==="string"?error:"Nie udało się wykonać operacji. Spróbuj ponownie."};
}
export interface SettingsApi {
  getSettings():Promise<AppSettings>; getAppVersion():Promise<string>;
  saveCompanySettings(input:SellerProfile):Promise<SellerProfile>; setCompanyLogo(source:string):Promise<SellerProfile>;
  removeCompanyLogo():Promise<SellerProfile>; getCompanyLogoDataUrl():Promise<string|null>;
  saveDocumentDefaults(input:DocumentDefaults):Promise<DocumentDefaults>; getNumberingSettings():Promise<NumberingSeriesSettings[]>;
  saveNumberingSettings(input:NumberingSeriesSettings):Promise<NumberingSeriesSettings>; previewNextNumber(document_type:"invoice"):Promise<NumberPreview>;
  saveAppearanceSettings(input:DocumentAppearanceSettings):Promise<DocumentAppearanceSettings>; saveAppPreferences(input:AppPreferences):Promise<AppPreferences>;
  createBackup(destination:string):Promise<BackupSummary>; inspectBackup(source:string):Promise<BackupSummary>;
  restoreBackup(source:string,confirmedReplace:boolean,expectedFingerprint:string):Promise<RestoreSummary>; getDataLocation():Promise<DataLocation>; openDataFolder():Promise<void>;
  chooseLogo():Promise<string|null>; chooseBackupDestination(defaultPath:string):Promise<string|null>; chooseBackupSource():Promise<string|null>;
}
export const settingsApi:SettingsApi={
  getSettings:()=>invoke("get_settings"),getAppVersion:()=>invoke("get_app_version"),
  saveCompanySettings:input=>invoke("save_company_settings",{input}),setCompanyLogo:source=>invoke("set_company_logo",{source}),removeCompanyLogo:()=>invoke("remove_company_logo"),getCompanyLogoDataUrl:()=>invoke("get_company_logo_data_url"),
  saveDocumentDefaults:input=>invoke("save_document_defaults",{input}),getNumberingSettings:()=>invoke("get_numbering_settings"),saveNumberingSettings:input=>invoke("save_numbering_settings",{input}),previewNextNumber:documentType=>invoke("preview_next_number",{documentType}),
  saveAppearanceSettings:input=>invoke("save_appearance_settings",{input}),saveAppPreferences:input=>invoke("save_app_preferences",{input}),
  createBackup:destination=>invoke("create_backup",{destination}),inspectBackup:source=>invoke("inspect_backup",{source}),restoreBackup:(source,confirmedReplace,expectedFingerprint)=>invoke("restore_backup",{source,confirmedReplace,expectedFingerprint}),getDataLocation:()=>invoke("get_data_location"),
  async openDataFolder(){const location=await this.getDataLocation();await openPath(location.app_data_dir);},
  chooseLogo:()=>open({multiple:false,directory:false,filters:[{name:"Obraz PNG / JPEG",extensions:["png","jpg","jpeg"]}]}),
  chooseBackupDestination:defaultPath=>save({defaultPath,filters:[{name:"Kopia DZG",extensions:["zip"]}]}),
  chooseBackupSource:()=>open({multiple:false,directory:false,filters:[{name:"Kopia DZG",extensions:["zip"]}]}),
};
