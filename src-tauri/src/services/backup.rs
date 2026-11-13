use crate::{db::{DatabaseState,migrations},errors::AppError};
use rusqlite::{Connection,OptionalExtension,OpenFlags};
use serde::{Serialize,Deserialize};
use sha2::{Digest,Sha256};
use std::{collections::{BTreeMap,HashSet},fs,io::{Read,Write},path::{Path,PathBuf}};

const MAX_ARCHIVE:u64=512*1024*1024;
const MAX_EXPANDED:u64=1024*1024*1024;
const MAX_ENTRY:u64=512*1024*1024;
const MAX_ENTRIES:usize=1024;
const MAX_MANIFEST:u64=1024*1024;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct BackupSummary {
    pub path:PathBuf, pub format_version:u16, pub schema_version:u32, pub app_version:String,
    pub created_at_utc:String, pub file_count:u32, pub size_bytes:u64,
}
#[derive(Debug,Serialize)]
pub struct RestoreSummary { pub restored_schema_version:u32, pub recovery_backup_path:PathBuf }
#[derive(Debug,Serialize)]
pub struct DataLocation { pub app_data_dir:PathBuf, pub database_path:PathBuf, pub assets_dir:PathBuf }
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct FileRecord {size_bytes:u64,sha256:String}
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {format_version:u16,schema_version:u32,app_version:String,created_at_utc:String,files:BTreeMap<String,FileRecord>}
struct ValidatedBackup {directory:tempfile::TempDir,summary:BackupSummary}
fn invalid(message:impl Into<String>)->AppError {AppError::new("invalid_backup",message)}
fn zip_error(error:zip::result::ZipError)->AppError {invalid(format!("Nieprawidłowe archiwum ZIP: {error}"))}
fn checksum(bytes:&[u8])->String {format!("{:x}",Sha256::digest(bytes))}
fn file_checksum(path:&Path)->Result<String,AppError> {
    let mut f=fs::File::open(path)?;let mut digest=Sha256::new();let mut b=[0;65536];
    loop {let n=f.read(&mut b)?;if n==0 {break;}digest.update(&b[..n]);} Ok(format!("{:x}",digest.finalize()))
}
fn safe_asset_name(name:&str)->bool {
    let upper=name.split('.').next().unwrap_or_default().to_ascii_uppercase();
    !name.is_empty() && name.len()<=200 && name.bytes().all(|b|b.is_ascii_alphanumeric()||b"-_.".contains(&b)) && !name.starts_with('.') && !name.ends_with('.')
        && !["CON","PRN","AUX","NUL","COM1","COM2","COM3","COM4","COM5","COM6","COM7","COM8","COM9","LPT1","LPT2","LPT3","LPT4","LPT5","LPT6","LPT7","LPT8","LPT9"].contains(&upper.as_str())
}
fn entry_allowed(name:&str)->bool {name=="manifest.json" || name=="database.sqlite" || name.strip_prefix("assets/").is_some_and(safe_asset_name)}
fn validate_entry_names<'a>(names:impl Iterator<Item=&'a str>)->Result<(),AppError> {
    let mut seen=HashSet::new();
    for name in names {if !entry_allowed(name)||!seen.insert(name.to_ascii_lowercase()) {return Err(invalid("Niedozwolona lub powtórzona ścieżka w kopii."));}} Ok(())
}
fn validate_limits(archive:u64,total:u64,entry:u64,count:usize,size:u64,compressed:u64)->Result<(),AppError> {
    if archive>MAX_ARCHIVE || total>MAX_EXPANDED || entry>MAX_ENTRY || count>MAX_ENTRIES || size>compressed.saturating_mul(100) {
        return Err(invalid("Przekroczono limit rozmiaru, liczby plików lub kompresji kopii."));} Ok(())
}
fn summary(path:&Path,m:&Manifest,size:u64)->BackupSummary {
    BackupSummary{path:path.into(),format_version:m.format_version,schema_version:m.schema_version,app_version:m.app_version.clone(),created_at_utc:m.created_at_utc.clone(),file_count:m.files.len() as u32,size_bytes:size}
}
pub fn get_data_location(state:&DatabaseState)->DataLocation {
    DataLocation {app_data_dir:state.data_dir().parent().unwrap().into(),database_path:state.data_dir().join("app.sqlite"),assets_dir:state.assets_dir().into()}
}
fn normalized_logo(connection:&Connection,assets:&Path,legacy_assets:Option<&Path>)->Result<(),AppError> {
    let reference:Option<String>=connection.query_row("SELECT value_json FROM settings WHERE key='company_logo_path'",[],|r|r.get(0)).optional()?;
    if let Some(json)=reference {
        let p:PathBuf=serde_json::from_str(&json).map_err(|_|invalid("Nieprawidłowe odwołanie do logo."))?;
        let name=p.file_name().and_then(|s|s.to_str()).ok_or_else(||invalid("Nieprawidłowa ścieżka logo."))?;
        if !safe_asset_name(name) || !(p==Path::new(name) || legacy_assets.is_some_and(|a|p.parent()==Some(a))) {
            return Err(invalid("Logo wskazuje poza zarządzany katalog zasobów."));
        }
        if !assets.join(name).is_file() {return Err(invalid("Kopia nie zawiera wskazanego logo."));}
        if p!=Path::new(name) {connection.execute("UPDATE settings SET value_json=?1 WHERE key='company_logo_path'",[serde_json::to_string(name).unwrap()])?;}
    } Ok(())
}
fn copy_assets(source:&Path,destination:&Path)->Result<(),AppError> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {let entry=entry?;let name=entry.file_name();
        if !entry.file_type()?.is_file() || !safe_asset_name(name.to_str().unwrap_or_default()) {return Err(invalid("Niedozwolony plik w katalogu zasobów."));}
        fs::copy(entry.path(),destination.join(name))?;
    }Ok(())
}
fn create_locked(state:&DatabaseState,c:&Connection,destination:&Path)->Result<BackupSummary,AppError> {
    if destination.extension().and_then(|s|s.to_str()).is_none_or(|x|!x.eq_ignore_ascii_case("zip")) {return Err(invalid("Kopia musi mieć rozszerzenie .zip."));}
    let parent=fs::canonicalize(destination.parent().ok_or_else(||invalid("Wybierz katalog kopii."))?)?;
    if parent.starts_with(fs::canonicalize(state.data_dir())?) {return Err(invalid("Zapisz kopię poza aktywnym katalogiem danych."));}
    let parent=destination.parent().unwrap();
    let stage=tempfile::tempdir()?;let database=stage.path().join("database.sqlite");
    let mut target=Connection::open(&database)?;
    rusqlite::backup::Backup::new(c,&mut target)?.run_to_completion(64,std::time::Duration::from_millis(5),None)?;
    let asset_dir=stage.path().join("assets");copy_assets(state.assets_dir(),&asset_dir)?;
    normalized_logo(&target,&asset_dir,Some(state.assets_dir()))?;
    let schema_version=target.query_row("PRAGMA user_version",[],|r|r.get(0))?;
    let created_at_utc=target.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now')",[],|r|r.get(0))?;
    drop(target);
    let mut files=BTreeMap::new();files.insert("database.sqlite".to_owned(),database);
    for entry in fs::read_dir(&asset_dir)? {let entry=entry?;files.insert(format!("assets/{}",entry.file_name().to_string_lossy()),entry.path());}
    let mut manifest=Manifest{format_version:1,schema_version,app_version:env!("CARGO_PKG_VERSION").into(),created_at_utc,files:BTreeMap::new()};
    let mut total=0;
    for (name,path) in &files {let size=fs::metadata(path)?.len();total+=size;validate_limits(0,total,size,files.len()+1,size,size)?;
        manifest.files.insert(name.clone(),FileRecord{size_bytes:size,sha256:file_checksum(path)?});}
    let archive_path=stage.path().join("archive.zip");let mut writer=zip::ZipWriter::new(fs::File::create(&archive_path)?);
    // Stored entries avoid enormous expansion ratios for valid sparse SQLite files.
    let options=zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name,path) in files {writer.start_file(name,options).map_err(zip_error)?;std::io::copy(&mut fs::File::open(path)?,&mut writer)?;}
    writer.start_file("manifest.json",options).map_err(zip_error)?;writer.write_all(&serde_json::to_vec(&manifest).map_err(|e|invalid(e.to_string()))?)?;
    writer.finish().map_err(zip_error)?.sync_all()?;
    validate_archive(&archive_path)?;
    let stem=destination.file_stem().and_then(|x|x.to_str()).ok_or_else(||invalid("Nieprawidłowa nazwa kopii."))?;
    for suffix in 0..10000 {
        let path=parent.join(if suffix==0 {format!("{stem}.zip")} else {format!("{stem}-{suffix:02}.zip")});
        match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file)=>{let result=(||{std::io::copy(&mut fs::File::open(&archive_path)?,&mut file)?;file.sync_all()?;Ok::<_,AppError>(())})();
                drop(file);if let Err(error)=result {let _=fs::remove_file(&path);return Err(error);}
                return Ok(summary(&path,&manifest,fs::metadata(&path)?.len()));},
            Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,
            Err(e)=>return Err(e.into()),
        }
    }Err(invalid("Nie można wybrać wolnej nazwy kopii."))
}
pub fn create_backup(state:&DatabaseState,destination:PathBuf)->Result<BackupSummary,AppError> {
    state.with_assets(||state.with_connection(|c|create_locked(state,c,&destination)))
}
fn validate_archive(source:&Path)->Result<ValidatedBackup,AppError> {
    let file=fs::File::open(source)?;let archive_size=file.metadata()?.len();validate_limits(archive_size,0,0,0,0,0)?;
    let mut archive=zip::ZipArchive::new(file).map_err(zip_error)?;
    validate_limits(archive_size,0,0,archive.len(),0,0)?;
    let names:Vec<_>=archive.file_names().map(str::to_owned).collect();validate_entry_names(names.iter().map(String::as_str))?;
    if !names.iter().any(|n|n=="manifest.json")||!names.iter().any(|n|n=="database.sqlite") {return Err(invalid("Niepełna kopia danych."));}
    let stage=tempfile::tempdir()?;let data=stage.path().join("data");fs::create_dir_all(data.join("assets"))?;
    let mut total:u64=0;let mut observed=BTreeMap::new();let mut manifest=None;
    for i in 0..archive.len() {
        let mut entry=archive.by_index(i).map_err(zip_error)?;
        if entry.is_dir() || entry.unix_mode().is_some_and(|m|m&0o170000!=0 && m&0o170000!=0o100000) {return Err(invalid("Kopia zawiera dowiązanie lub plik specjalny."));}
        total=total.checked_add(entry.size()).ok_or_else(||invalid("Przekroczony rozmiar kopii."))?;
        validate_limits(archive_size,total,entry.size(),names.len(),entry.size(),entry.compressed_size())?;
        let name=entry.name().to_owned();
        if name=="manifest.json" {
            if entry.size()>MAX_MANIFEST {return Err(invalid("Manifest kopii jest zbyt duży."));}
            let mut bytes=Vec::new();entry.take(MAX_MANIFEST+1).read_to_end(&mut bytes)?;
            if bytes.len() as u64>MAX_MANIFEST {return Err(invalid("Manifest kopii jest zbyt duży."));}
            manifest=Some(serde_json::from_slice::<Manifest>(&bytes).map_err(|_|invalid("Nieprawidłowy manifest kopii."))?);
        } else {
            let path=if name=="database.sqlite" {data.join("app.sqlite")} else {data.join(&name)};
            let declared=entry.size();let mut output=fs::OpenOptions::new().write(true).create_new(true).open(&path)?;
            let actual=std::io::copy(&mut entry.by_ref().take(declared+1),&mut output)?;
            if actual!=declared {return Err(invalid("Rozmiar pliku jest niezgodny z deklaracją."));}drop(output);
            observed.insert(name,FileRecord{size_bytes:actual,sha256:file_checksum(&path)?});
        }
    }
    let m=manifest.ok_or_else(||invalid("Brak manifestu kopii."))?;
    if m.format_version!=1 || m.schema_version<1 || m.schema_version>migrations::LATEST_VERSION {return Err(invalid("Nieobsługiwana wersja formatu lub bazy danych."));}
    if m.app_version.is_empty() || m.created_at_utc.len()!=20 || !m.created_at_utc.ends_with('Z') || m.files.len()!=observed.len() {return Err(invalid("Niepełny manifest kopii."));}
    for (name,expected) in &m.files {let actual=observed.get(name).ok_or_else(||invalid("Brak wymaganego pliku kopii."))?;
        if expected.size_bytes!=actual.size_bytes || expected.sha256!=actual.sha256 {return Err(invalid("Suma kontrolna lub rozmiar pliku jest niezgodny."));}}
    let mut connection=Connection::open_with_flags(data.join("app.sqlite"),OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    let integrity:String=connection.query_row("PRAGMA integrity_check",[],|r|r.get(0)).map_err(|_|invalid("Uszkodzona baza SQLite."))?;
    let actual_version:u32=connection.query_row("PRAGMA user_version",[],|r|r.get(0))?;
    if integrity!="ok"||actual_version!=m.schema_version {return Err(invalid("Baza nie przeszła kontroli integralności lub wersji."));}
    migrations::run(&mut connection)?;
    for sql in ["SELECT key,value_json FROM settings LIMIT 1","SELECT document_type,formatted_number FROM issued_numbers LIMIT 1","SELECT reset_policy,counter_period,next_counter FROM numbering_series LIMIT 1"] {connection.prepare(sql)?;}
    normalized_logo(&connection,&data.join("assets"),None)?;drop(connection);
    Ok(ValidatedBackup{directory:stage,summary:summary(source,&m,archive_size)})
}
pub fn inspect_backup(source:&Path)->Result<BackupSummary,AppError> {Ok(validate_archive(source)?.summary)}
fn swap_data(state:&DatabaseState,connection:&mut Option<Connection>,staged:&Path,after_swap:impl FnOnce()->Result<(),AppError>)->Result<(),AppError> {
    let rollback=state.data_dir().with_file_name("data-rollback");
    if rollback.exists() {return Err(AppError::new("restore_error","Pozostał katalog odzyskiwania z poprzedniej operacji. Nie zmieniono danych."));}
    if let Some(c)=connection.take() {if let Err((c,e))=c.close() {*connection=Some(c);return Err(e.into());}}
    if let Err(e)=fs::rename(state.data_dir(),&rollback) {*connection=Some(state.reopen()?);return Err(e.into());}
    let result=(|| {fs::rename(staged,state.data_dir())?;after_swap()?;*connection=Some(state.reopen()?);Ok::<_,AppError>(())})();
    if let Err(e)=result {
        // Keep the previous complete data set until the new connection has opened successfully.
        if state.data_dir().exists() {fs::rename(state.data_dir(),staged)?;}
        fs::rename(&rollback,state.data_dir())?;*connection=Some(state.reopen()?);return Err(e);
    }
    // A cleanup failure must never make a completed restore look like a failed replacement.
    // The old set is also preserved by the verified recovery archive.
    fs::remove_dir_all(&rollback).map_err(|e|AppError::new("restore_cleanup_error",format!("Dane przywrócono; nie można usunąć katalogu odzyskiwania: {e}")))?;
    Ok(())
}
pub fn restore_backup(state:&DatabaseState,source:PathBuf,confirmed_replace:bool)->Result<RestoreSummary,AppError> {
    state.with_assets(||restore_locked(state,source,confirmed_replace))
}
fn restore_locked(state:&DatabaseState,source:PathBuf,confirmed_replace:bool)->Result<RestoreSummary,AppError> {
    if !confirmed_replace {return Err(AppError::new("confirmation_required","Potwierdź zastąpienie wszystkich lokalnych danych."));}
    let validated=validate_archive(&source)?;
    // Copy to the app-data volume before any live-data mutation; directory renames stay atomic.
    let stage=tempfile::tempdir_in(state.data_dir().parent().unwrap())?;let data=stage.path().join("data");fs::create_dir_all(&data)?;
    fs::copy(validated.directory.path().join("data/app.sqlite"),data.join("app.sqlite"))?;
    copy_assets(&validated.directory.path().join("data/assets"),&data.join("assets"))?;
    state.with_exclusive(|connection| {
        let c=connection.as_ref().ok_or_else(||AppError::new("database_closed","Baza danych jest zamknięta."))?;
        let recovery=create_locked(state,c,&state.recovery_dir().join("before-restore.zip"))?;
        inspect_backup(&recovery.path)?;
        swap_data(state,connection,&data,||Ok(()))?;
        Ok(RestoreSummary{restored_schema_version:migrations::LATEST_VERSION,recovery_backup_path:recovery.path})
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::settings::SellerProfile,services::settings};
    fn database()->(tempfile::TempDir,DatabaseState) {let d=tempfile::tempdir().unwrap();let s=DatabaseState::open(d.path().join("profile")).unwrap();(d,s)}
    fn company(s:&DatabaseState,name:&str) {settings::save_company_settings(s,SellerProfile{name:Some(name.into()),..Default::default()}).unwrap();}
    fn fingerprint(s:&DatabaseState)->(Vec<u8>,Vec<u8>) {(fs::read(s.data_dir().join("app.sqlite")).unwrap(),fs::read(s.assets_dir().join("company-logo-test.png")).unwrap_or_default())}
    fn archive(path:&Path,entries:Vec<(&str,Vec<u8>)>) {let mut z=zip::ZipWriter::new(fs::File::create(path).unwrap()); for (name,bytes) in entries {z.start_file(name,zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored)).unwrap();z.write_all(&bytes).unwrap();}z.finish().unwrap();}
    fn entries(path:&Path)->Vec<(String,Vec<u8>)> {let mut z=zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();(0..z.len()).map(|i|{let mut f=z.by_index(i).unwrap();let name=f.name().to_owned();let mut b=Vec::new();f.read_to_end(&mut b).unwrap();(name,b)}).collect()}
    fn rewrite(path:&Path,items:Vec<(String,Vec<u8>)>) {archive(path,items.iter().map(|(n,b)|(n.as_str(),b.clone())).collect());}
    #[test] fn round_trip_contains_database_and_logo() {
        let (d,s)=database();company(&s,"Original");fs::write(s.assets_dir().join("company-logo-test.png"),b"logo").unwrap();
        let backup=create_backup(&s,d.path().join("copy.zip")).unwrap();assert_eq!(inspect_backup(&backup.path).unwrap().file_count,2);
        company(&s,"Changed");fs::write(s.assets_dir().join("company-logo-test.png"),b"changed").unwrap();
        let result=restore_backup(&s,backup.path,true).unwrap();assert!(result.recovery_backup_path.exists());
        assert_eq!(settings::get_settings(&s).unwrap().company.name.as_deref(),Some("Original"));assert_eq!(fs::read(s.assets_dir().join("company-logo-test.png")).unwrap(),b"logo");
    }
    #[test] fn repeated_backups_get_unique_names_without_overwrite() {
        let(d,s)=database();let p=d.path().join("copy.zip");let a=create_backup(&s,p.clone()).unwrap();let original=fs::read(&a.path).unwrap();
        assert_eq!(create_backup(&s,p).unwrap().path,d.path().join("copy-01.zip"));assert_eq!(fs::read(a.path).unwrap(),original);
    }
    #[test] fn rejects_truncated_archive() {let(d,s)=database();let p=d.path().join("bad.zip");fs::write(&p,b"PK\x03\x04").unwrap();assert!(inspect_backup(&p).is_err());assert_unchanged(&s,&p);}
    fn assert_unchanged(s:&DatabaseState,p:&Path) {let before=fingerprint(s);assert!(restore_backup(s,p.into(),true).is_err());assert_eq!(before,fingerprint(s));}
    #[test] fn rejects_duplicate_entries() {
        let(d,s)=database();let p=create_backup(&s,d.path().join("copy.zip")).unwrap().path;let mut items=entries(&p);items.push(items[0].clone());
        // ZIP writer rejects duplicates too; the validator additionally checks entry identity.
        assert!(validate_entry_names(["manifest.json","manifest.json"].into_iter()).is_err());
        let mut items=entries(&p);items.push(("assets/../database.sqlite".into(),vec![]));rewrite(&p,items);assert_unchanged(&s,&p);
    }
    #[test] fn rejects_path_traversal() {let(d,s)=database();for name in ["../escape","assets/../../escape","/absolute","C:/absolute","assets\\escape","assets/x:y","assets/CON","assets/a."] {let p=d.path().join("bad.zip");archive(&p,vec![(name,vec![1])]);assert_unchanged(&s,&p);}}
    #[test] fn rejects_bad_checksum() {let(d,s)=database();let p=create_backup(&s,d.path().join("copy.zip")).unwrap().path;let mut x=entries(&p);x.iter_mut().find(|(n,_)|n=="database.sqlite").unwrap().1[0]^=1;rewrite(&p,x);assert_unchanged(&s,&p);}
    #[test] fn rejects_newer_schema() {let(d,s)=database();let p=create_backup(&s,d.path().join("copy.zip")).unwrap().path;let mut x=entries(&p);let manifest=x.iter_mut().find(|(n,_)|n=="manifest.json").unwrap();let mut m:serde_json::Value=serde_json::from_slice(&manifest.1).unwrap();m["schema_version"]=999.into();manifest.1=serde_json::to_vec(&m).unwrap();rewrite(&p,x);assert_unchanged(&s,&p);}
    #[test] fn rejects_each_archive_limit_boundary() {
        for (compressed,expanded,entry,count,ratio_size,ratio_compressed) in [(MAX_ARCHIVE+1,1,1,1,1,1),(1,MAX_EXPANDED+1,1,1,1,1),(1,1,MAX_ENTRY+1,1,1,1),(1,1,1,MAX_ENTRIES+1,1,1),(1,1,1,1,101,1)] {
            assert!(validate_limits(compressed,expanded,entry,count,ratio_size,ratio_compressed).is_err());
        }
        assert!(validate_limits(MAX_ARCHIVE,MAX_EXPANDED,MAX_ENTRY,MAX_ENTRIES,100,1).is_ok());
    }
    #[test] fn rejects_corrupt_sqlite_before_confirmation() {
        let(d,s)=database();let p=create_backup(&s,d.path().join("copy.zip")).unwrap().path;let mut x=entries(&p);let bytes=b"not sqlite".to_vec();x.iter_mut().find(|(n,_)|n=="database.sqlite").unwrap().1=bytes.clone();
        let m=x.iter_mut().find(|(n,_)|n=="manifest.json").unwrap();let mut manifest:Manifest=serde_json::from_slice(&m.1).unwrap();manifest.files.insert("database.sqlite".into(),FileRecord{size_bytes:bytes.len() as u64,sha256:checksum(&bytes)});m.1=serde_json::to_vec(&manifest).unwrap();rewrite(&p,x);assert!(inspect_backup(&p).is_err());assert_unchanged(&s,&p);
    }
    #[test] fn restore_requires_confirmation() {let(d,s)=database();let p=create_backup(&s,d.path().join("copy.zip")).unwrap().path;let before=fingerprint(&s);assert!(restore_backup(&s,p,false).is_err());assert_eq!(before,fingerprint(&s));}
    #[test] fn failed_restore_restores_database_and_assets() {
        let(d,s)=database();company(&s,"Saved");fs::write(s.assets_dir().join("company-logo-test.png"),b"saved").unwrap();let p=create_backup(&s,d.path().join("copy.zip")).unwrap().path;
        company(&s,"Active");fs::write(s.assets_dir().join("company-logo-test.png"),b"active").unwrap();let before=fingerprint(&s);
        let validated=validate_archive(&p).unwrap();s.with_exclusive(|conn|{assert!(swap_data(&s,conn,&validated.directory.path().join("data"),||Err(AppError::new("injected","failure"))).is_err());Ok(())}).unwrap();
        assert_eq!(before,fingerprint(&s));assert_eq!(settings::get_settings(&s).unwrap().company.name.as_deref(),Some("Active"));
    }
    #[test] fn unlisted_assets_are_rejected() {let(d,s)=database();let p=create_backup(&s,d.path().join("copy.zip")).unwrap().path;let mut x=entries(&p);x.push(("assets/unlisted.png".into(),vec![1]));rewrite(&p,x);assert_unchanged(&s,&p);}
    #[test] fn real_logo_survives_restore_into_a_different_profile() {
        let (d,s)=database();let logo=d.path().join("logo.png");image::RgbaImage::new(2,2).save(&logo).unwrap();settings::set_company_logo(&s,logo).unwrap();
        let p=create_backup(&s,d.path().join("copy.zip")).unwrap().path;let (_d2,s2)=database();restore_backup(&s2,p,true).unwrap();
        assert!(settings::get_company_logo_data_url(&s2).unwrap().unwrap().starts_with("data:image/png;base64,"));
    }
}
