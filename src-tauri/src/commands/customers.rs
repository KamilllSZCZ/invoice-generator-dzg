use crate::db::DatabaseState;
use crate::errors::AppError;
use crate::models::customers::{Customer, CustomerInput};
use crate::services::customers;
use tauri::State;

#[tauri::command]
pub fn list_customers(
    state: State<'_, DatabaseState>,
    query: String,
) -> Result<Vec<Customer>, AppError> {
    customers::list_customers(&state, &query)
}

#[tauri::command]
pub fn create_customer(
    state: State<'_, DatabaseState>,
    input: CustomerInput,
) -> Result<Customer, AppError> {
    customers::create_customer(&state, input)
}

#[tauri::command]
pub fn update_customer(
    state: State<'_, DatabaseState>,
    id: i64,
    input: CustomerInput,
) -> Result<Customer, AppError> {
    customers::update_customer(&state, id, input)
}

#[tauri::command]
pub fn delete_customer(state: State<'_, DatabaseState>, id: i64) -> Result<(), AppError> {
    customers::delete_customer(&state, id)
}
