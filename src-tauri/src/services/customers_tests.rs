use super::*;
use crate::models::settings::SellerProfile;
use crate::services::{backup, settings};

fn database() -> (tempfile::TempDir, DatabaseState) {
    let directory = tempfile::tempdir().unwrap();
    let state = DatabaseState::open(directory.path().join("profile")).unwrap();
    (directory, state)
}

fn input(name: &str) -> CustomerInput {
    CustomerInput {
        name: name.into(),
        ..Default::default()
    }
}

#[test]
fn normalizes_optional_fields_and_persists_after_reopen() {
    let (directory, state) = database();
    let saved = create_customer(
        &state,
        CustomerInput {
            name: "  Test — Zażółć gęślą jaźń  ".into(),
            tax_id: Some(" DE-TEST ".into()),
            city: Some(" Łódź ".into()),
            email: Some(" test@example.test ".into()),
            address: Some("  ".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(saved.details.name, "Test — Zażółć gęślą jaźń");
    assert_eq!(saved.details.tax_id.as_deref(), Some("DE-TEST"));
    assert_eq!(saved.details.email.as_deref(), Some("test@example.test"));
    assert_eq!(saved.details.address, None);
    assert_eq!(saved.details.country, None);
    drop(state);
    let reopened = DatabaseState::open(directory.path().join("profile")).unwrap();
    assert_eq!(list_customers(&reopened, "").unwrap(), vec![saved]);
}

#[test]
fn update_preserves_identity_and_refreshes_search() {
    let (_directory, state) = database();
    let saved = create_customer(&state, input("Pierwsza nazwa")).unwrap();
    let changed = update_customer(
        &state,
        saved.id,
        CustomerInput {
            name: "Nowa nazwa".into(),
            phone: Some("+48 TEST".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(changed.id, saved.id);
    assert_eq!(changed.details.phone.as_deref(), Some("+48 TEST"));
    assert!(list_customers(&state, "Pierwsza").unwrap().is_empty());
    assert_eq!(list_customers(&state, "NOWA").unwrap(), vec![changed]);
    assert_eq!(
        update_customer(&state, 99999, input("Brak"))
            .unwrap_err()
            .code,
        "customer_not_found"
    );
}

#[test]
fn validation_failures_never_partially_write() {
    let (_directory, state) = database();
    let original = create_customer(&state, input("Poprawny wpis")).unwrap();
    let invalid = CustomerInput {
        name: " \t\n ".into(),
        email: Some("a@b@example.test".into()),
        notes: Some("ą".repeat(2001)),
        ..Default::default()
    };
    let error = update_customer(&state, original.id, invalid.clone()).unwrap_err();
    assert_eq!(error.code, "validation_error");
    let fields = error.field_errors.unwrap();
    for name in ["name", "email", "notes"] {
        assert!(fields.get(name).is_some());
    }
    assert!(create_customer(&state, invalid).is_err());
    assert!(create_customer(&state, input(&"ą".repeat(201))).is_err());
    assert_eq!(list_customers(&state, "").unwrap(), vec![original]);
}

#[test]
fn search_is_unicode_case_insensitive_and_treats_sql_characters_literally() {
    let (_directory, state) = database();
    let saved = create_customer(
        &state,
        CustomerInput {
            name: "Test 100%_Łódź".into(),
            tax_id: Some("TEST-123".into()),
            city: Some("Żory".into()),
            ..Default::default()
        },
    )
    .unwrap();
    create_customer(&state, input("Drugi wpis")).unwrap();
    for query in ["łÓDŹ", "żORY", "test-123", "%_", "  łódź  "] {
        assert_eq!(list_customers(&state, query).unwrap(), vec![saved.clone()]);
    }
    assert!(list_customers(&state, "' OR 1=1 --").unwrap().is_empty());
    assert_eq!(list_customers(&state, "").unwrap().len(), 2);
    assert!(list_customers(&state, &"a".repeat(201)).is_err());
}

#[test]
fn deletes_only_requested_customer_and_does_not_reuse_identity() {
    let (_directory, state) = database();
    let first = create_customer(&state, input("Pierwszy")).unwrap();
    let second = create_customer(&state, input("Drugi")).unwrap();
    delete_customer(&state, first.id).unwrap();
    assert_eq!(list_customers(&state, "").unwrap(), vec![second.clone()]);
    assert_eq!(
        delete_customer(&state, first.id).unwrap_err().code,
        "customer_not_found"
    );
    assert_eq!(
        delete_customer(&state, -1).unwrap_err().code,
        "customer_not_found"
    );
    let third = create_customer(&state, input("Trzeci")).unwrap();
    assert!(third.id > second.id);
}

#[test]
fn linked_document_blocks_deletion_and_edit_does_not_change_snapshot_after_reopen() {
    let (directory, state) = database();
    let saved = create_customer(&state, input("Nazwa przy wystawieniu")).unwrap();
    let snapshot = serde_json::to_string(&saved.details).unwrap();
    state
        .with_connection(|connection| {
            // A fixture exercises the FK contract for the future document module.
            connection.execute_batch(
                "CREATE TABLE document_fixture (
            customer_id INTEGER NOT NULL REFERENCES customers(id) ON DELETE RESTRICT,
            buyer_snapshot TEXT NOT NULL
        )",
            )?;
            connection.execute(
                "INSERT INTO document_fixture VALUES (?1, ?2)",
                (saved.id, &snapshot),
            )?;
            Ok(())
        })
        .unwrap();
    drop(state);
    let reopened = DatabaseState::open(directory.path().join("profile")).unwrap();
    update_customer(&reopened, saved.id, input("Zmieniona nazwa")).unwrap();
    assert_eq!(
        delete_customer(&reopened, saved.id).unwrap_err().code,
        "customer_in_use"
    );
    reopened
        .with_connection(|connection| {
            let actual: String =
                connection.query_row("SELECT buyer_snapshot FROM document_fixture", [], |row| {
                    row.get(0)
                })?;
            assert_eq!(actual, snapshot);
            Ok(())
        })
        .unwrap();
    assert_eq!(list_customers(&reopened, "").unwrap().len(), 1);
}

#[test]
fn backups_restore_all_customer_fields_and_enable_foreign_keys_after_swap() {
    let (directory, state) = database();
    let original = create_customer(
        &state,
        CustomerInput {
            name: "Test kopii".into(),
            tax_id: Some("ID-TEST".into()),
            address: Some("Adres testowy".into()),
            postal_code: Some("TEST".into()),
            city: Some("Miejscowość testowa".into()),
            country: Some("Kraj testowy".into()),
            email: Some("test@example.test".into()),
            phone: Some("TEST".into()),
            notes: Some("Uwagi testowe".into()),
        },
    )
    .unwrap();
    let copy = backup::create_backup(&state, directory.path().join("customers.zip")).unwrap();
    assert_eq!(copy.schema_version, 3);
    let summary = backup::inspect_backup(&copy.path).unwrap();
    delete_customer(&state, original.id).unwrap();
    create_customer(&state, input("Stan przed przywróceniem")).unwrap();
    let restored = backup::restore_backup(&state, copy.path, true, &summary.fingerprint).unwrap();
    assert!(restored.recovery_backup_path.is_file());
    assert_eq!(list_customers(&state, "").unwrap(), vec![original]);
    state
        .with_connection(|connection| {
            let enabled: bool =
                connection.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
            assert!(enabled);
            Ok(())
        })
        .unwrap();
}

#[test]
fn version_two_backup_migrates_without_losing_settings() {
    let (directory, state) = database();
    settings::save_company_settings(
        &state,
        SellerProfile {
            name: Some("Profil sprzed migracji".into()),
            ..Default::default()
        },
    )
    .unwrap();
    state
        .with_connection(|connection| {
            connection.execute_batch("DROP TABLE customers; PRAGMA user_version = 2;")?;
            Ok(())
        })
        .unwrap();
    let copy = backup::create_backup(&state, directory.path().join("version-two.zip")).unwrap();
    assert_eq!(copy.schema_version, 2);
    let summary = backup::inspect_backup(&copy.path).unwrap();
    backup::restore_backup(&state, copy.path, true, &summary.fingerprint).unwrap();
    assert!(list_customers(&state, "").unwrap().is_empty());
    assert_eq!(
        settings::get_settings(&state)
            .unwrap()
            .company
            .name
            .as_deref(),
        Some("Profil sprzed migracji")
    );
    create_customer(&state, input("Nowa kartoteka po migracji")).unwrap();
}

#[test]
fn backup_creation_rejects_invalid_customer_data() {
    let (directory, state) = database();
    let saved = create_customer(&state, input("Spójny wpis")).unwrap();
    state
        .with_connection(|connection| {
            connection.execute(
                "UPDATE customers SET email = 'invalid-email' WHERE id = ?1",
                [saved.id],
            )?;
            Ok(())
        })
        .unwrap();
    let error =
        backup::create_backup(&state, directory.path().join("invalid-customer.zip")).unwrap_err();
    assert_eq!(error.code, "validation_error");
}
