pub(crate) mod migrations;

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
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        migrations::run(&mut connection)?;

        Ok(Self {
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

    pub(crate) fn with_assets<T>(&self, operation:impl FnOnce()->Result<T,AppError>)->Result<T,AppError> {
        let _guard=self.assets_lock.lock().map_err(|_|AppError::new("assets_lock_error","Nie można uzyskać dostępu do zasobów."))?;
        operation()
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub(crate) fn with_exclusive<T>(&self, operation: impl FnOnce(&mut Option<Connection>) -> Result<T,AppError>) -> Result<T,AppError> {
        let mut guard=self.connection.lock().map_err(|_| AppError::new("database_lock_error","Nie można uzyskać dostępu do danych."))?;
        operation(&mut guard)
    }

    pub(crate) fn reopen(&self) -> Result<Connection,AppError> {
        let mut c=Connection::open(self.data_dir.join("app.sqlite"))?;
        c.busy_timeout(std::time::Duration::from_secs(5))?;
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
