use crate::db::DatabaseState;
use crate::errors::AppError;
use crate::models::settings::{
    AppPreferences, AppSettings, DocumentAppearanceSettings, DocumentDefaults, SellerProfile,
};
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<'_, DatabaseState>) -> Result<AppSettings, AppError> {
    crate::services::settings::get_settings(&state)
}

#[tauri::command]
pub fn get_app_version() -> Result<String, AppError> {
    Ok(env!("CARGO_PKG_VERSION").to_owned())
}

#[tauri::command]
pub fn save_company_settings(
    state: State<'_, DatabaseState>,
    input: SellerProfile,
) -> Result<SellerProfile, AppError> {
    crate::services::settings::save_company_settings(&state, input)
}

#[tauri::command]
pub fn set_company_logo(
    state: State<'_, DatabaseState>,
    source: PathBuf,
) -> Result<SellerProfile, AppError> {
    crate::services::settings::set_company_logo(&state, source)
}

#[tauri::command]
pub fn remove_company_logo(state: State<'_, DatabaseState>) -> Result<SellerProfile, AppError> {
    crate::services::settings::remove_company_logo(&state)
}

#[tauri::command]
pub fn get_company_logo_data_url(
    state: State<'_, DatabaseState>,
) -> Result<Option<String>, AppError> {
    crate::services::settings::get_company_logo_data_url(&state)
}

#[tauri::command]
pub fn save_document_defaults(
    state: State<'_, DatabaseState>,
    input: DocumentDefaults,
) -> Result<DocumentDefaults, AppError> {
    crate::services::settings::save_document_defaults(&state, input)
}

#[tauri::command]
pub fn save_appearance_settings(
    state: State<'_, DatabaseState>,
    input: DocumentAppearanceSettings,
) -> Result<DocumentAppearanceSettings, AppError> {
    crate::services::settings::save_appearance_settings(&state, input)
}

#[tauri::command]
pub fn save_app_preferences(
    state: State<'_, DatabaseState>,
    input: AppPreferences,
) -> Result<AppPreferences, AppError> {
    crate::services::settings::save_preferences(&state, input)
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_app_version_returns_package_version() {
        assert_eq!(super::get_app_version().unwrap(), env!("CARGO_PKG_VERSION"));
    }
}
