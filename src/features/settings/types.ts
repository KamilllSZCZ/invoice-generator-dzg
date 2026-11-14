export interface SellerProfile {
  name:string|null; address:string|null; postal_code:string|null; city:string|null; country:string|null;
  tax_id:string|null; registration_id:string|null; email:string|null; phone:string|null; website:string|null; bank_account:string|null;
}
export interface DocumentDefaults { document_type:"invoice"; currency:string|null; payment_method:string|null; payment_term_days:number|null; issue_place:string|null; notes:string|null; }
export interface NumberingSeriesSettings { document_type:"invoice"; prefix:string; format:string; reset_policy:"yearly"|"monthly"|"never"|null; next_counter:number; }
export interface DocumentAppearanceSettings { template:string|null; accent_color:string|null; show_logo:boolean|null; footer:string|null; show_address:boolean|null; show_email:boolean|null; show_phone:boolean|null; show_website:boolean|null; show_bank_account:boolean|null; }
export interface AppPreferences { language:string|null; theme:"system"|"light"|"dark"|null; density:"comfortable"|"compact"|null; }
export interface AppSettings { company:SellerProfile; document_defaults:DocumentDefaults; numbering:NumberingSeriesSettings[]; appearance:DocumentAppearanceSettings; preferences:AppPreferences; }
export interface NumberPreview { document_type:"invoice"; formatted_number:string; period_key:string; counter:number; }
export interface BackupSummary { path:string; format_version:number; schema_version:number; app_version:string; created_at_utc:string; file_count:number; size_bytes:number; fingerprint:string; }
export interface RestoreSummary { restored_schema_version:number; recovery_backup_path:string; }
export interface DataLocation { app_data_dir:string; database_path:string; assets_dir:string; }
export type FieldErrors=Record<string,string>;
export interface SettingsError { code:string; message:string; field_errors?:FieldErrors; }
