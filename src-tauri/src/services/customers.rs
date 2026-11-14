use crate::db::{DatabaseState, customers as repository};
use crate::errors::AppError;
use crate::models::customers::{Customer, CustomerInput};
use rusqlite::Connection;
use serde_json::{Map, Value};

fn invalid(fields: Map<String, Value>) -> AppError {
    AppError::new("validation_error", "Sprawdź dane kontrahenta.")
        .with_field_errors(Value::Object(fields))
}

fn not_found() -> AppError {
    AppError::new(
        "customer_not_found",
        "Nie znaleziono kontrahenta. Odśwież listę.",
    )
}

fn normalize(mut input: CustomerInput) -> Result<CustomerInput, AppError> {
    input.name = input.name.trim().to_owned();
    let mut fields = Map::new();
    if input.name.is_empty() {
        fields.insert("name".into(), "Podaj nazwę kontrahenta.".into());
    } else if input.name.chars().count() > 200 {
        fields.insert(
            "name".into(),
            "Nazwa może mieć maksymalnie 200 znaków.".into(),
        );
    }
    for (key, value, limit) in [
        ("tax_id", &mut input.tax_id, 40),
        ("address", &mut input.address, 300),
        ("postal_code", &mut input.postal_code, 30),
        ("city", &mut input.city, 100),
        ("country", &mut input.country, 100),
        ("email", &mut input.email, 254),
        ("phone", &mut input.phone, 50),
        ("notes", &mut input.notes, 2000),
    ] {
        *value = value
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_owned);
        if value.as_deref().is_some_and(|v| v.chars().count() > limit) {
            fields.insert(key.into(), format!("Maksymalnie {limit} znaków.").into());
        }
    }
    if let Some(email) = &input.email {
        let valid = !email.chars().any(char::is_whitespace)
            && email.split_once('@').is_some_and(|(local, domain)| {
                !local.is_empty()
                    && !domain.contains('@')
                    && domain.contains('.')
                    && !domain.starts_with('.')
                    && !domain.ends_with('.')
            });
        if !valid {
            fields.insert("email".into(), "Podaj prawidłowy adres e-mail.".into());
        }
    }
    if !fields.is_empty() {
        return Err(invalid(fields));
    }
    Ok(input)
}

fn search_key(input: &CustomerInput) -> String {
    [
        input.name.as_str(),
        input.tax_id.as_deref().unwrap_or(""),
        input.city.as_deref().unwrap_or(""),
    ]
    .join("\n")
    .to_lowercase()
}

pub fn list_customers(state: &DatabaseState, query: &str) -> Result<Vec<Customer>, AppError> {
    if query.chars().count() > 200 {
        return Err(AppError::new(
            "validation_error",
            "Wyszukiwanie może mieć maksymalnie 200 znaków.",
        ));
    }
    let query = query.trim().to_lowercase();
    state.with_connection(|connection| Ok(repository::list(connection, &query)?))
}

pub fn create_customer(state: &DatabaseState, input: CustomerInput) -> Result<Customer, AppError> {
    let input = normalize(input)?;
    state.with_connection(|connection| {
        let transaction = connection.transaction()?;
        let id = repository::insert(&transaction, &input, &search_key(&input))?;
        let saved = repository::get(&transaction, id)?.ok_or_else(not_found)?;
        transaction.commit()?;
        Ok(saved)
    })
}

pub fn update_customer(
    state: &DatabaseState,
    id: i64,
    input: CustomerInput,
) -> Result<Customer, AppError> {
    let input = normalize(input)?;
    state.with_connection(|connection| {
        let transaction = connection.transaction()?;
        if !repository::update(&transaction, id, &input, &search_key(&input))? {
            return Err(not_found());
        }
        let saved = repository::get(&transaction, id)?.ok_or_else(not_found)?;
        transaction.commit()?;
        Ok(saved)
    })
}

pub fn delete_customer(state: &DatabaseState, id: i64) -> Result<(), AppError> {
    state.with_connection(|connection| match repository::delete(connection, id) {
        Ok(true) => Ok(()),
        Ok(false) => Err(not_found()),
        Err(rusqlite::Error::SqliteFailure(error, message))
            if error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY
                || (error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_TRIGGER
                    && message.as_deref() == Some("FOREIGN KEY constraint failed")) =>
        {
            Err(AppError::new(
                "customer_in_use",
                "Nie można usunąć kontrahenta powiązanego z dokumentem.",
            ))
        }
        Err(error) => Err(error.into()),
    })
}

pub(crate) fn validate_database(connection: &Connection) -> Result<(), AppError> {
    for customer in repository::list(connection, "")? {
        let normalized = normalize(customer.details.clone())?;
        if normalized != customer.details
            || repository::search_text(connection, customer.id)? != search_key(&normalized)
        {
            return Err(AppError::new(
                "invalid_backup",
                "Kopia zawiera niespójne dane kontrahenta.",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "customers_tests.rs"]
mod tests;
