mod migrations;

use crate::errors::AppError;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct DatabaseState {
    connection: Mutex<Option<Connection>>,
    data_dir: PathBuf,
    assets_dir: PathBuf,
    recovery_dir: PathBuf,
}

impl DatabaseState {
    pub fn open(data_dir: PathBuf) -> Result<Self, AppError> {
        let active_data_dir = data_dir.join("data");
        let assets_dir = active_data_dir.join("assets");
        let recovery_dir = data_dir.join("recovery");
        fs::create_dir_all(&assets_dir)?;
        fs::create_dir_all(&recovery_dir)?;

        let database_path = active_data_dir.join("app.sqlite");
        let mut connection = Connection::open(database_path)?;
        migrations::run(&mut connection)?;

        Ok(Self {
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

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn assets_dir(&self) -> &Path {
        &self.assets_dir
    }

    pub fn recovery_dir(&self) -> &Path {
        &self.recovery_dir
    }
}
