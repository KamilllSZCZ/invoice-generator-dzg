use crate::db::DatabaseState;
use crate::errors::AppError;
use crate::models::settings::{
    AppPreferences, AppSettings, DocumentAppearanceSettings, DocumentDefaults, SellerProfile,
};
use base64::Engine;
use image::{ImageDecoder, ImageFormat, ImageReader, Limits};
use rusqlite::{OptionalExtension, Transaction};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LOGO_BYTES: u64 = 10 * 1024 * 1024;
const MAX_LOGO_PIXELS: u64 = 25_000_000;
const COMPANY_LOGO_KEY: &str = "company_logo_path";
const IMAGE_DECODE_LIMIT: u64 = MAX_LOGO_PIXELS * 4;

pub fn get_settings(state: &DatabaseState) -> Result<AppSettings, AppError> {
    state.with_connection(|connection| {
        let company = read_json(connection, "company")?.unwrap_or_default();
        let document_defaults = read_json(connection, "document_defaults")?.unwrap_or_default();
        let appearance = read_json(connection, "appearance")?.unwrap_or_default();
        let preferences = read_json(connection, "preferences")?.unwrap_or_default();
        let mut statement = connection.prepare(
            "SELECT document_type, prefix, format, counter_period, next_counter FROM numbering_series ORDER BY document_type",
        ).map_err(AppError::from)?;
        let numbering = statement.query_map([], |row| {
            Ok(crate::models::settings::NumberingSeriesSettings {
                document_type: crate::models::settings::DocumentTypeKey::new(row.get::<_, String>(0)?),
                prefix: row.get(1)?,
                format: row.get(2)?,
                reset_policy: row.get(3)?,
                next_counter: u64::try_from(row.get::<_, i64>(4)?).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(4, -1))?,
            })
        }).map_err(AppError::from)?.collect::<Result<Vec<_>, _>>().map_err(AppError::from)?;
        Ok(AppSettings { company, document_defaults, numbering, appearance, preferences })
    })
}

pub fn save_settings(state: &DatabaseState, settings: &AppSettings) -> Result<(), AppError> {
    validate_company(&settings.company)?;
    validate_appearance(&settings.appearance)?;
    validate_preferences(&settings.preferences)?;
    state.with_connection(|connection| {
        let transaction = connection.transaction().map_err(AppError::from)?;
        write_json(&transaction, "company", &settings.company)?;
        write_json(&transaction, "document_defaults", &settings.document_defaults)?;
        write_json(&transaction, "appearance", &settings.appearance)?;
        write_json(&transaction, "preferences", &settings.preferences)?;
        transaction.execute("DELETE FROM numbering_series", []).map_err(AppError::from)?;
        for series in &settings.numbering {
            let next_counter = i64::try_from(series.next_counter)
                .map_err(|_| validation_error("numbering", "Numbering counter is too large"))?;
            transaction.execute(
                "INSERT INTO numbering_series (document_type, prefix, format, counter_period, next_counter) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![series.document_type.as_str(), series.prefix, series.format, series.reset_policy, next_counter],
            ).map_err(AppError::from)?;
        }
        transaction.commit().map_err(AppError::from)
    })
}

pub fn save_company_settings(
    state: &DatabaseState,
    input: SellerProfile,
) -> Result<SellerProfile, AppError> {
    validate_company(&input)?;
    state.with_connection(|connection| {
        let transaction = connection.transaction().map_err(AppError::from)?;
        write_json(&transaction, "company", &input)?;
        transaction.commit().map_err(AppError::from)?;
        Ok(input)
    })
}

pub fn save_document_defaults(
    state: &DatabaseState,
    input: DocumentDefaults,
) -> Result<DocumentDefaults, AppError> {
    state.with_connection(|connection| {
        let transaction = connection.transaction().map_err(AppError::from)?;
        write_json(&transaction, "document_defaults", &input)?;
        transaction.commit().map_err(AppError::from)?;
        Ok(input)
    })
}

pub fn save_appearance_settings(
    state: &DatabaseState,
    input: DocumentAppearanceSettings,
) -> Result<DocumentAppearanceSettings, AppError> {
    validate_appearance(&input)?;
    state.with_connection(|connection| {
        let transaction = connection.transaction().map_err(AppError::from)?;
        write_json(&transaction, "appearance", &input)?;
        transaction.commit().map_err(AppError::from)?;
        Ok(input)
    })
}

pub fn save_preferences(
    state: &DatabaseState,
    input: AppPreferences,
) -> Result<AppPreferences, AppError> {
    validate_preferences(&input)?;
    state.with_connection(|connection| {
        let transaction = connection.transaction().map_err(AppError::from)?;
        write_json(&transaction, "preferences", &input)?;
        transaction.commit().map_err(AppError::from)?;
        Ok(input)
    })
}

pub fn set_company_logo(state: &DatabaseState, source: PathBuf) -> Result<SellerProfile, AppError> {
    let company = get_company_profile(state)?;
    let source_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let extension = Path::new(source_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let bytes = read_limited(&source)?;
    let (format, dimensions) = decode_logo(&bytes)?;
    let expected_extension = match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        _ => {
            return Err(validation_error(
                "logo",
                "Only PNG and JPEG logos are supported",
            ));
        }
    };
    if extension != expected_extension && !(format == ImageFormat::Jpeg && extension == "jpeg") {
        return Err(validation_error(
            "logo",
            "Logo file extension does not match its image format",
        ));
    }
    let pixels = u64::from(dimensions.0) * u64::from(dimensions.1);
    if pixels > MAX_LOGO_PIXELS {
        return Err(validation_error(
            "logo",
            "Logo exceeds the 25-megapixel limit",
        ));
    }

    let filename = generated_logo_name(expected_extension);
    let destination = state.assets_dir().join(&filename);
    fs::write(&destination, bytes)?;
    let old_path = match set_logo_reference(state, &destination) {
        Ok(path) => path,
        Err(error) => {
            let _ = fs::remove_file(&destination);
            return Err(error);
        }
    };
    remove_unreferenced_managed_logo(state, old_path.as_deref());
    Ok(company)
}

pub fn remove_company_logo(state: &DatabaseState) -> Result<SellerProfile, AppError> {
    let company = get_company_profile(state)?;
    let old_path = state.with_connection(|connection| {
        let transaction = connection.transaction().map_err(AppError::from)?;
        let old_path: Option<PathBuf> = transaction
            .query_row(
                "SELECT value_json FROM settings WHERE key = ?1",
                [COMPANY_LOGO_KEY],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(AppError::from)?
            .and_then(|value| serde_json::from_str(&value).ok());
        transaction
            .execute("DELETE FROM settings WHERE key = ?1", [COMPANY_LOGO_KEY])
            .map_err(AppError::from)?;
        transaction.commit().map_err(AppError::from)?;
        Ok(old_path)
    })?;
    remove_unreferenced_managed_logo(state, old_path.as_deref());
    Ok(company)
}

pub fn get_company_logo_data_url(state: &DatabaseState) -> Result<Option<String>, AppError> {
    let path = get_logo_reference(state)?;
    let Some(path) = path else {
        return Ok(None);
    };
    if !is_managed_logo_path(state, &path) {
        return Err(AppError::new(
            "settings_error",
            "Stored company logo path is not managed by the application",
        ));
    }
    let bytes = read_limited(&path)?;
    let (format, _) = decode_logo(&bytes)?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let mime = match (extension, format) {
        ("png", ImageFormat::Png) => "image/png",
        ("jpg", ImageFormat::Jpeg) => "image/jpeg",
        _ => {
            return Err(AppError::new(
                "settings_error",
                "Stored logo has an unsupported or mismatched format",
            ));
        }
    };
    Ok(Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )))
}

fn get_company_profile(state: &DatabaseState) -> Result<SellerProfile, AppError> {
    state.with_connection(|connection| Ok(read_json(connection, "company")?.unwrap_or_default()))
}

fn validate_company(company: &SellerProfile) -> Result<(), AppError> {
    if let Some(email) = &company.email {
        let trimmed = email.trim();
        let valid = trimmed == email
            && !email.chars().any(char::is_whitespace)
            && email.split_once('@').is_some_and(|(local, domain)| {
                !local.is_empty()
                    && domain.contains('.')
                    && !domain.starts_with('.')
                    && !domain.ends_with('.')
            });
        if !valid {
            return Err(validation_error("email", "Enter a valid email address"));
        }
    }
    Ok(())
}

fn validate_appearance(_appearance: &DocumentAppearanceSettings) -> Result<(), AppError> {
    Ok(())
}

fn validate_preferences(preferences: &AppPreferences) -> Result<(), AppError> {
    if let Some(theme) = &preferences.theme {
        if !["light", "dark", "system"].contains(&theme.as_str()) {
            return Err(validation_error(
                "theme",
                "Theme must be light, dark, or system",
            ));
        }
    }
    Ok(())
}

fn validation_error(field: &str, message: &str) -> AppError {
    AppError::new("validation_error", "One or more settings are invalid")
        .with_field_errors(serde_json::json!({ field: message }))
}

fn read_json<T: serde::de::DeserializeOwned>(
    connection: &rusqlite::Connection,
    key: &str,
) -> Result<Option<T>, AppError> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()
        .map_err(AppError::from)?;
    value
        .map(|json| {
            serde_json::from_str(&json).map_err(|_| {
                AppError::new(
                    "settings_error",
                    format!("Stored {key} settings are invalid"),
                )
            })
        })
        .transpose()
}

fn write_json<T: serde::Serialize>(
    transaction: &Transaction<'_>,
    key: &str,
    value: &T,
) -> Result<(), AppError> {
    let mut value = serde_json::to_value(value)
        .map_err(|_| AppError::new("serialization_error", "Could not serialize settings"))?;
    let value_object = value.as_object_mut().ok_or_else(|| {
        AppError::new(
            "serialization_error",
            "Settings payload must be a JSON object",
        )
    })?;

    let existing: Option<String> = transaction
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()
        .map_err(AppError::from)?;
    if let Some(existing) = existing {
        let mut existing: serde_json::Value = serde_json::from_str(&existing).map_err(|_| {
            AppError::new(
                "settings_error",
                format!("Stored {key} settings are invalid"),
            )
        })?;
        let existing_object = existing.as_object_mut().ok_or_else(|| {
            AppError::new(
                "settings_error",
                format!("Stored {key} settings must be a JSON object"),
            )
        })?;
        merge_json_objects(existing_object, value_object);
        value = existing;
    }

    let json = serde_json::to_string(&value)
        .map_err(|_| AppError::new("serialization_error", "Could not serialize settings"))?;
    transaction.execute("INSERT INTO settings (key, value_json) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json", [key, &json]).map_err(AppError::from)?;
    Ok(())
}

fn merge_json_objects(
    existing: &mut serde_json::Map<String, serde_json::Value>,
    updates: &serde_json::Map<String, serde_json::Value>,
) {
    for (key, update) in updates {
        match (existing.get_mut(key), update) {
            (Some(serde_json::Value::Object(existing)), serde_json::Value::Object(update)) => {
                merge_json_objects(existing, update);
            }
            _ => {
                existing.insert(key.clone(), update.clone());
            }
        }
    }
}

fn read_limited(path: &Path) -> Result<Vec<u8>, AppError> {
    let file = fs::File::open(path)?;
    let size = file.metadata()?.len();
    if size > MAX_LOGO_BYTES {
        return Err(validation_error(
            "logo",
            "Logo file exceeds the 10 MiB limit",
        ));
    }
    let mut bytes = Vec::with_capacity(usize::try_from(size).unwrap_or(0));
    file.take(MAX_LOGO_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_LOGO_BYTES {
        return Err(validation_error(
            "logo",
            "Logo file exceeds the 10 MiB limit",
        ));
    }
    Ok(bytes)
}

fn decode_logo(bytes: &[u8]) -> Result<(ImageFormat, (u32, u32)), AppError> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| validation_error("logo", "Logo must be a valid PNG or JPEG image"))?;
    let format = reader
        .format()
        .ok_or_else(|| validation_error("logo", "Logo format could not be determined"))?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg) {
        return Err(validation_error(
            "logo",
            "Only PNG and JPEG logos are supported",
        ));
    }
    let mut decoder = reader
        .into_decoder()
        .map_err(|_| validation_error("logo", "Logo image could not be decoded"))?;
    let dimensions = decoder.dimensions();
    if u64::from(dimensions.0) * u64::from(dimensions.1) > MAX_LOGO_PIXELS {
        return Err(validation_error(
            "logo",
            "Logo exceeds the 25-megapixel limit",
        ));
    }
    let mut limits = Limits::default();
    limits.max_alloc = Some(IMAGE_DECODE_LIMIT);
    decoder
        .set_limits(limits)
        .map_err(|_| validation_error("logo", "Logo exceeds the bounded decoding limit"))?;
    let buffer_size = decoder.total_bytes();
    let mut buffer = vec![
        0;
        usize::try_from(buffer_size)
            .map_err(|_| validation_error("logo", "Logo image is too large"))?
    ];
    decoder
        .read_image(&mut buffer)
        .map_err(|_| validation_error("logo", "Logo image could not be decoded"))?;
    Ok((format, dimensions))
}

fn generated_logo_name(extension: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("company-logo-{}-{nanos}.{extension}", std::process::id())
}

fn get_logo_reference(state: &DatabaseState) -> Result<Option<PathBuf>, AppError> {
    state.with_connection(|connection| {
        let value: Option<String> = connection
            .query_row(
                "SELECT value_json FROM settings WHERE key = ?1",
                [COMPANY_LOGO_KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(AppError::from)?;
        Ok(value.and_then(|json| serde_json::from_str::<PathBuf>(&json).ok()))
    })
}

fn set_logo_reference(state: &DatabaseState, path: &Path) -> Result<Option<PathBuf>, AppError> {
    state.with_connection(|connection| {
        let transaction = connection.transaction().map_err(AppError::from)?;
        let old_path: Option<String> = transaction.query_row("SELECT value_json FROM settings WHERE key = ?1", [COMPANY_LOGO_KEY], |row| row.get::<_, String>(0)).optional().map_err(AppError::from)?;
        let encoded_path = serde_json::to_string(path).map_err(|_| AppError::new("serialization_error", "Could not serialize logo path"))?;
        transaction.execute("INSERT INTO settings (key, value_json) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json", [COMPANY_LOGO_KEY, &encoded_path]).map_err(AppError::from)?;
        transaction.commit().map_err(AppError::from)?;
        Ok(old_path.and_then(|value| serde_json::from_str(&value).ok()))
    })
}

fn remove_unreferenced_managed_logo(state: &DatabaseState, old_path: Option<&Path>) {
    let Some(path) = old_path else {
        return;
    };
    if is_managed_logo_path(state, path)
        && get_logo_reference(state).ok().flatten().as_deref() != Some(path)
    {
        let _ = fs::remove_file(path);
    }
}

fn is_managed_logo_path(state: &DatabaseState, path: &Path) -> bool {
    path.parent() == Some(state.assets_dir())
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.starts_with("company-logo-")
                    && matches!(
                        path.extension().and_then(|ext| ext.to_str()),
                        Some("png" | "jpg")
                    )
            })
}

#[cfg(test)]
mod tests {
    use crate::db::DatabaseState;
    use crate::errors::AppError;
    use crate::models::settings::{AppPreferences, AppSettings, SellerProfile};
    use std::fs;

    fn database() -> (tempfile::TempDir, DatabaseState) {
        let directory = tempfile::tempdir().unwrap();
        let state = DatabaseState::open(directory.path().to_path_buf()).unwrap();
        (directory, state)
    }

    #[test]
    fn settings_round_trip_after_reopen() {
        let directory = tempfile::tempdir().unwrap();
        let settings = AppSettings {
            company: SellerProfile {
                name: Some("Northwind".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        {
            let state = DatabaseState::open(directory.path().to_path_buf()).unwrap();
            super::save_settings(&state, &settings).unwrap();
        }
        let reopened = DatabaseState::open(directory.path().to_path_buf()).unwrap();
        assert_eq!(super::get_settings(&reopened).unwrap(), settings);
    }

    #[test]
    fn settings_section_saves_preserve_unknown_fields_and_update_known_values() {
        let (_directory, state) = database();
        state.with_connection(|connection| {
            connection.execute_batch(
                r#"
                INSERT INTO settings (key, value_json) VALUES
                    ('company', '{"name":"Before","future_company":{"nested":"keep"},"future_root":true}'),
                    ('document_defaults', '{"document_type":"invoice","future_defaults":{"nested":7}}'),
                    ('appearance', '{"template":"classic","future_appearance":[1,2]}'),
                    ('preferences', '{"theme":"light","future_preferences":{"nested":false}}');
                "#,
            ).map_err(AppError::from)?;
            Ok(())
        }).unwrap();

        let loaded = super::get_settings(&state).unwrap();
        assert_eq!(loaded.company.name.as_deref(), Some("Before"));

        super::save_company_settings(
            &state,
            SellerProfile {
                name: Some("Updated".into()),
                ..Default::default()
            },
        )
        .unwrap();
        super::save_document_defaults(
            &state,
            crate::models::settings::DocumentDefaults {
                currency: Some("PLN".into()),
                ..Default::default()
            },
        )
        .unwrap();
        super::save_appearance_settings(
            &state,
            crate::models::settings::DocumentAppearanceSettings {
                template: Some("modern".into()),
                ..Default::default()
            },
        )
        .unwrap();
        super::save_preferences(
            &state,
            AppPreferences {
                theme: Some("dark".into()),
                ..Default::default()
            },
        )
        .unwrap();

        state
            .with_connection(|connection| {
                for (key, known_path, known_value, unknown_path, unknown_value) in [
                    (
                        "company",
                        "name",
                        serde_json::json!("Updated"),
                        "future_company.nested",
                        serde_json::json!("keep"),
                    ),
                    (
                        "document_defaults",
                        "currency",
                        serde_json::json!("PLN"),
                        "future_defaults.nested",
                        serde_json::json!(7),
                    ),
                    (
                        "appearance",
                        "template",
                        serde_json::json!("modern"),
                        "future_appearance",
                        serde_json::json!([1, 2]),
                    ),
                    (
                        "preferences",
                        "theme",
                        serde_json::json!("dark"),
                        "future_preferences.nested",
                        serde_json::json!(false),
                    ),
                ] {
                    let stored: String = connection
                        .query_row(
                            "SELECT value_json FROM settings WHERE key = ?1",
                            [key],
                            |row| row.get(0),
                        )
                        .map_err(AppError::from)?;
                    let value: serde_json::Value = serde_json::from_str(&stored)
                        .map_err(|error| AppError::new("settings_error", error.to_string()))?;
                    assert_eq!(value[known_path], known_value, "known field for {key}");
                    let unknown = unknown_path
                        .split('.')
                        .fold(&value, |current, segment| &current[segment]);
                    assert_eq!(unknown, &unknown_value, "unknown field for {key}");
                }
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn settings_section_save_does_not_replace_corrupted_json() {
        let (_directory, state) = database();
        let corrupted = "{not valid JSON";
        state
            .with_connection(|connection| {
                connection
                    .execute(
                        "INSERT INTO settings (key, value_json) VALUES ('company', ?1)",
                        [corrupted],
                    )
                    .map_err(AppError::from)?;
                Ok(())
            })
            .unwrap();

        let error = super::save_company_settings(
            &state,
            SellerProfile {
                name: Some("Must not overwrite corruption".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "settings_error");
        state
            .with_connection(|connection| {
                let stored: String = connection
                    .query_row(
                        "SELECT value_json FROM settings WHERE key = 'company'",
                        [],
                        |row| row.get(0),
                    )
                    .map_err(AppError::from)?;
                assert_eq!(stored, corrupted);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn empty_unregistered_seller_is_valid() {
        let (_directory, state) = database();
        super::save_company_settings(&state, SellerProfile::default()).unwrap();
        assert_eq!(
            super::get_settings(&state).unwrap().company,
            SellerProfile::default()
        );
    }

    #[test]
    fn invalid_supplied_email_returns_field_error() {
        let (_directory, state) = database();
        let existing = SellerProfile {
            name: Some("Original".into()),
            ..Default::default()
        };
        super::save_company_settings(&state, existing.clone()).unwrap();
        let error = super::save_company_settings(
            &state,
            SellerProfile {
                name: Some("Must not be partially saved".into()),
                email: Some("not-an-email".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "validation_error");
        assert!(error.field_errors.unwrap()["email"].is_string());
        assert_eq!(super::get_settings(&state).unwrap().company, existing);
    }

    #[test]
    fn invalid_theme_returns_field_error() {
        let (_directory, state) = database();
        let error = super::save_preferences(
            &state,
            AppPreferences {
                theme: Some("ultraviolet".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "validation_error");
        assert!(error.field_errors.unwrap()["theme"].is_string());
    }

    #[test]
    fn rejects_logo_with_mismatched_extension_or_oversize_pixels() {
        let (_directory, state) = database();
        let mismatch = tempfile::Builder::new().suffix(".jpg").tempfile().unwrap();
        fs::write(mismatch.path(), tiny_png()).unwrap();
        assert!(super::set_company_logo(&state, mismatch.path().to_path_buf()).is_err());

        let oversized = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        fs::write(oversized.path(), png_with_dimensions(5001, 5000)).unwrap();
        assert!(super::set_company_logo(&state, oversized.path().to_path_buf()).is_err());

        let oversized_file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        fs::write(oversized_file.path(), vec![0; 10 * 1024 * 1024 + 1]).unwrap();
        assert!(super::set_company_logo(&state, oversized_file.path().to_path_buf()).is_err());
    }

    #[test]
    fn valid_logo_is_stored_as_managed_asset_and_removed_safely() {
        let (_directory, state) = database();
        let source = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        fs::write(source.path(), tiny_png()).unwrap();

        super::set_company_logo(&state, source.path().to_path_buf()).unwrap();
        let data_url = super::get_company_logo_data_url(&state).unwrap().unwrap();
        assert!(data_url.starts_with("data:image/png;base64,"));
        let assets = fs::read_dir(state.assets_dir())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(assets.len(), 1);

        super::remove_company_logo(&state).unwrap();
        assert!(super::get_company_logo_data_url(&state).unwrap().is_none());
        assert!(fs::read_dir(state.assets_dir()).unwrap().next().is_none());
    }

    #[test]
    fn save_company_settings_does_not_change_issued_snapshot() {
        let (_directory, state) = database();
        let original = vec![0, 255, 17, 42];
        state.with_connection(|connection| {
            connection.execute_batch(
                "CREATE TABLE fixture_issued_snapshot (id INTEGER PRIMARY KEY, payload BLOB NOT NULL);",
            ).map_err(AppError::from)?;
            connection.execute(
                "INSERT INTO fixture_issued_snapshot (id, payload) VALUES (1, ?1)",
                [&original],
            ).map_err(AppError::from)?;
            Ok(())
        }).unwrap();

        super::save_company_settings(
            &state,
            SellerProfile {
                name: Some("Changed seller".into()),
                ..Default::default()
            },
        )
        .unwrap();

        state
            .with_connection(|connection| {
                let payload: Vec<u8> = connection
                    .query_row(
                        "SELECT payload FROM fixture_issued_snapshot WHERE id = 1",
                        [],
                        |row| row.get(0),
                    )
                    .map_err(AppError::from)?;
                assert_eq!(payload, original);
                Ok(())
            })
            .unwrap();
    }

    fn tiny_png() -> Vec<u8> {
        vec![
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
            8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 156, 99, 248, 207,
            192, 240, 31, 0, 5, 0, 1, 255, 137, 153, 61, 29, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66,
            96, 130,
        ]
    }

    fn png_with_dimensions(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82];
        bytes.extend(width.to_be_bytes());
        bytes.extend(height.to_be_bytes());
        bytes.extend([8, 6, 0, 0, 0, 0, 0, 0, 0]);
        bytes
    }
}
