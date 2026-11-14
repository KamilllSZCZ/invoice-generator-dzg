import { invoke } from "@tauri-apps/api/core";
import type { Customer, CustomerInput, CustomerFieldErrors } from "./types";

export interface CustomersApi {
  listCustomers(query: string): Promise<Customer[]>;
  createCustomer(input: CustomerInput): Promise<Customer>;
  updateCustomer(id: number, input: CustomerInput): Promise<Customer>;
  deleteCustomer(id: number): Promise<void>;
}

export const customersApi: CustomersApi = {
  listCustomers: query => invoke("list_customers", { query }),
  createCustomer: input => invoke("create_customer", { input }),
  updateCustomer: (id, input) => invoke("update_customer", { id, input }),
  deleteCustomer: id => invoke("delete_customer", { id }),
};

export function customerError(error: unknown): { message: string; field_errors?: CustomerFieldErrors } {
  if (typeof error === "object" && error !== null && "message" in error) {
    const result = error as { message: unknown; field_errors?: CustomerFieldErrors };
    return { message: String(result.message), field_errors: result.field_errors };
  }
  return { message: typeof error === "string" ? error : "Nie udało się wykonać operacji na kontrahencie. Spróbuj ponownie." };
}
