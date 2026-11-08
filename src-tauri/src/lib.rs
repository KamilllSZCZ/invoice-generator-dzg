pub mod commands;
pub mod db;
pub mod errors;
pub mod models;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;

    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let state = db::DatabaseState::open(app_data_dir).map_err(|error| {
                std::io::Error::other(format!("Could not initialize local database: {error}"))
            })?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::get_app_version,
            commands::settings::save_company_settings,
            commands::settings::set_company_logo,
            commands::settings::remove_company_logo,
            commands::settings::get_company_logo_data_url,
            commands::settings::save_document_defaults,
            commands::settings::save_appearance_settings,
            commands::settings::save_app_preferences,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Dual Zone Garage");
}
