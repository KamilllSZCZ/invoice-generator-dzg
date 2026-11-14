use crate::models::customers::{Customer, CustomerInput};
use rusqlite::{Connection, OptionalExtension, Row, named_params};

const COLUMNS: &str = "id, name, tax_id, address, postal_code, city, country, email, phone, notes";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Customer> {
    Ok(Customer {
        id: row.get(0)?,
        details: CustomerInput {
            name: row.get(1)?,
            tax_id: row.get(2)?,
            address: row.get(3)?,
            postal_code: row.get(4)?,
            city: row.get(5)?,
            country: row.get(6)?,
            email: row.get(7)?,
            phone: row.get(8)?,
            notes: row.get(9)?,
        },
    })
}

pub(crate) fn list(connection: &Connection, query: &str) -> rusqlite::Result<Vec<Customer>> {
    let mut statement = connection.prepare(&format!(
        "SELECT {COLUMNS} FROM customers WHERE instr(search_text, ?1) > 0
         ORDER BY name COLLATE NOCASE, id"
    ))?;
    statement.query_map([query], from_row)?.collect()
}

pub(crate) fn get(connection: &Connection, id: i64) -> rusqlite::Result<Option<Customer>> {
    connection
        .query_row(
            &format!("SELECT {COLUMNS} FROM customers WHERE id = ?1"),
            [id],
            from_row,
        )
        .optional()
}

pub(crate) fn insert(
    connection: &Connection,
    input: &CustomerInput,
    search_text: &str,
) -> rusqlite::Result<i64> {
    connection.execute(
        "INSERT INTO customers (name, tax_id, address, postal_code, city, country, email, phone, notes, search_text)
         VALUES (:name, :tax_id, :address, :postal_code, :city, :country, :email, :phone, :notes, :search)",
        named_params! {
            ":name": input.name, ":tax_id": input.tax_id, ":address": input.address,
            ":postal_code": input.postal_code, ":city": input.city, ":country": input.country,
            ":email": input.email, ":phone": input.phone, ":notes": input.notes,
            ":search": search_text,
        },
    )?;
    Ok(connection.last_insert_rowid())
}

pub(crate) fn update(
    connection: &Connection,
    id: i64,
    input: &CustomerInput,
    search_text: &str,
) -> rusqlite::Result<bool> {
    let changed = connection.execute(
        "UPDATE customers SET name = :name, tax_id = :tax_id, address = :address,
         postal_code = :postal_code, city = :city, country = :country, email = :email,
         phone = :phone, notes = :notes, search_text = :search WHERE id = :id",
        named_params! {
            ":id": id, ":name": input.name, ":tax_id": input.tax_id, ":address": input.address,
            ":postal_code": input.postal_code, ":city": input.city, ":country": input.country,
            ":email": input.email, ":phone": input.phone, ":notes": input.notes,
            ":search": search_text,
        },
    )?;
    Ok(changed == 1)
}

pub(crate) fn delete(connection: &Connection, id: i64) -> rusqlite::Result<bool> {
    Ok(connection.execute("DELETE FROM customers WHERE id = ?1", [id])? == 1)
}

pub(crate) fn search_text(connection: &Connection, id: i64) -> rusqlite::Result<String> {
    connection.query_row(
        "SELECT search_text FROM customers WHERE id = ?1",
        [id],
        |row| row.get(0),
    )
}
