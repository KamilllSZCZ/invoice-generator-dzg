export interface CustomerInput {
  name: string;
  tax_id: string | null;
  address: string | null;
  postal_code: string | null;
  city: string | null;
  country: string | null;
  email: string | null;
  phone: string | null;
  notes: string | null;
}

export interface Customer extends CustomerInput {
  id: number;
}

export type CustomerFieldErrors = Partial<Record<keyof CustomerInput | "_form", string>>;

export function emptyCustomer(): CustomerInput {
  return {
    name: "", tax_id: null, address: null, postal_code: null, city: null,
    country: null, email: null, phone: null, notes: null,
  };
}

export function customerInput(customer: Customer): CustomerInput {
  const { id: _id, ...input } = customer;
  return input;
}
