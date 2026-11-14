pub(crate) mod migrations;
pub(crate) mod customers;

use crate::errors::AppError;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct DatabaseState {
    assets_lock: Mutex<()>,
    connection: Mutex<Option<Connection>>,
    data_dir: PathBuf,
    assets_dir: PathBuf,
    recovery_dir: PathBuf,
    // Drop the process lock after closing SQLite and all other state resources.
    instance_lock: Option<fs::File>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_app_instance_can_own_a_profile_until_its_connection_closes() {
        let directory = tempfile::tempdir().unwrap();
        let state = DatabaseState::open_single_instance(directory.path().into()).unwrap();
        assert_eq!(
            DatabaseState::open_single_instance(directory.path().into())
                .err()
                .unwrap()
                .code,
            "app_already_running"
        );
        drop(state);
        let reopened = DatabaseState::open_single_instance(directory.path().into()).unwrap();
        reopened
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO settings(key,value_json) VALUES ('probe','{}')",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
    }
}

impl DatabaseState {
    pub fn open_single_instance(root: PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(&root)?;
        let lock = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join("instance.lock"))?;
        lock.try_lock().map_err(|_| {
            AppError::new(
                "app_already_running",
                "Aplikacja korzystająca z tych danych jest już uruchomiona.",
            )
        })?;
        let mut state = Self::open(root)?;
        state.instance_lock = Some(lock);
        Ok(state)
    }
    pub fn open(data_dir: PathBuf) -> Result<Self, AppError> {
        let active_data_dir = data_dir.join("data");
        let assets_dir = active_data_dir.join("assets");
        let recovery_dir = data_dir.join("recovery");
        let rollback = data_dir.join("data-rollback");
        if rollback.exists() {
            if rollback.join("restore-committed").is_file() {
                if !active_data_dir.is_dir() {
                    return Err(AppError::new(
                        "recovery_required",
                        "Brak zatwierdzonego katalogu danych. Zachowano katalog odzyskiwania.",
                    ));
                }
                fs::remove_dir_all(&rollback)?;
            } else {
                fs::create_dir_all(&recovery_dir)?;
                let interrupted = recovery_dir.join(format!(
                    "interrupted-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                ));
                let had_active = active_data_dir.exists();
                if had_active {
                    fs::rename(&active_data_dir, &interrupted)?;
                }
                if let Err(error) = fs::rename(&rollback, &active_data_dir) {
                    if had_active {
                        fs::rename(&interrupted, &active_data_dir)?;
                    }
                    return Err(error.into());
                }
            }
        }
        fs::create_dir_all(&assets_dir)?;
        fs::create_dir_all(&recovery_dir)?;

        let database_path = active_data_dir.join("app.sqlite");
        let mut connection = Connection::open(database_path)?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.pragma_update(None, "foreign_keys", true)?;
        migrations::run(&mut connection)?;

        Ok(Self {
            instance_lock: None,
            assets_lock: Mutex::new(()),
            connection: Mutex::new(Some(connection)),
            data_dir: active_data_dir,
            assets_dir,
            recovery_dir,
        })
    }

    pub fn with_connection<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let mut guard = self
            .connection
            .lock()
            .map_err(|_| AppError::new("database_lock_error", "Database lock is poisoned"))?;
        let connection = guard
            .as_mut()
            .ok_or_else(|| AppError::new("database_closed", "Database connection is closed"))?;
        operation(connection)
    }

    pub(crate) fn with_assets<T>(
        &self,
        operation: impl FnOnce() -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let _guard = self.assets_lock.lock().map_err(|_| {
            AppError::new("assets_lock_error", "Nie można uzyskać dostępu do zasobów.")
        })?;
        operation()
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub(crate) fn with_exclusive<T>(
        &self,
        operation: impl FnOnce(&mut Option<Connection>) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let mut guard = self.connection.lock().map_err(|_| {
            AppError::new(
                "database_lock_error",
                "Nie można uzyskać dostępu do danych.",
            )
        })?;
        operation(&mut guard)
    }

    pub(crate) fn reopen(&self) -> Result<Connection, AppError> {
        let mut c = Connection::open(self.data_dir.join("app.sqlite"))?;
        c.busy_timeout(std::time::Duration::from_secs(5))?;
        c.pragma_update(None, "foreign_keys", true)?;
        migrations::run(&mut c)?;
        Ok(c)
    }

    pub fn assets_dir(&self) -> &Path {
        &self.assets_dir
    }

    pub fn recovery_dir(&self) -> &Path {
        &self.recovery_dir
    }
}
