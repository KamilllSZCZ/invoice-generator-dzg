use crate::{db::DatabaseState,errors::AppError,services::backup::{self,BackupSummary,RestoreSummary,DataLocation}};
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn create_backup(state:State<'_,DatabaseState>,destination:PathBuf)->Result<BackupSummary,AppError> {backup::create_backup(&state,destination)}
#[tauri::command]
pub async fn inspect_backup(source:PathBuf)->Result<BackupSummary,AppError> {backup::inspect_backup(&source)}
#[tauri::command]
pub async fn restore_backup(state:State<'_,DatabaseState>,source:PathBuf,confirmed_replace:bool)->Result<RestoreSummary,AppError> {backup::restore_backup(&state,source,confirmed_replace)}
#[tauri::command]
pub fn get_data_location(state:State<'_,DatabaseState>)->Result<DataLocation,AppError> {Ok(backup::get_data_location(&state))}
