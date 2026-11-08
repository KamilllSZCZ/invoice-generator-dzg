use crate::errors::AppError;
use rusqlite::Connection;

const MIGRATIONS: &[(i64, &str)] = &[(1, include_str!("../../migrations/0001_settings.sql"))];

pub(super) fn run(connection: &mut Connection) -> Result<(), AppError> {
    let current_version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let latest_version = MIGRATIONS.last().map(|(version, _)| *version).unwrap_or(0);
    if current_version > latest_version {
        return Err(AppError::new(
            "unsupported_schema_version",
            format!(
                "Database schema version {current_version} is newer than supported version {latest_version}"
            ),
        ));
    }

    for (version, sql) in MIGRATIONS {
        if *version <= current_version {
            continue;
        }
        let transaction = connection.transaction()?;
        transaction.execute_batch(sql)?;
        transaction.pragma_update(None, "user_version", version)?;
        transaction.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db::DatabaseState;
    use crate::errors::AppError;

    #[test]
    fn rejects_database_version_newer_than_supported_migrations() {
        let mut connection = rusqlite::Connection::open_in_memory().unwrap();
        connection.pragma_update(None, "user_version", 2).unwrap();

        let error = super::run(&mut connection).unwrap_err();

        assert_eq!(error.code, "unsupported_schema_version");
    }

    #[test]
    fn fresh_database_migrates_to_version_1() {
        let temp_dir = tempfile::tempdir().unwrap();
        let state = DatabaseState::open(temp_dir.path().to_path_buf()).unwrap();

        state
            .with_connection(|connection| {
                let version: i64 = connection
                    .query_row("PRAGMA user_version", [], |row| row.get(0))
                    .map_err(AppError::from)?;
                assert_eq!(version, 1);
                for table in ["settings", "numbering_series", "issued_numbers"] {
                    let exists: bool = connection
                        .query_row(
                            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                            [table],
                            |row| row.get(0),
                        )
                        .map_err(AppError::from)?;
                    assert!(exists, "missing table {table}");
                }
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn reopen_does_not_rerun_migration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let data_dir = temp_dir.path().to_path_buf();
        let state = DatabaseState::open(data_dir.clone()).unwrap();
        state
            .with_connection(|connection| {
                connection
                    .execute("CREATE TABLE migration_probe (value TEXT NOT NULL)", [])
                    .map_err(AppError::from)?;
                Ok(())
            })
            .unwrap();
        drop(state);

        let reopened = DatabaseState::open(data_dir).unwrap();
        reopened
            .with_connection(|connection| {
                let exists: bool = connection
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'migration_probe')",
                        [],
                        |row| row.get(0),
                    )
                    .map_err(AppError::from)?;
                assert!(exists);
                let version: i64 = connection
                    .query_row("PRAGMA user_version", [], |row| row.get(0))
                    .map_err(AppError::from)?;
                assert_eq!(version, 1);
                Ok(())
            })
            .unwrap();
    }
}
