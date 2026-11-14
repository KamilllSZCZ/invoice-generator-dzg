import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import { CustomersPage } from "./CustomersPage";
import type { CustomersApi } from "./api";
import { emptyCustomer } from "./types";
import type { Customer, CustomerInput } from "./types";

afterEach(() => vi.restoreAllMocks());

function customer(id = 1, name = "Kontrahent testowy"): Customer {
  return { id, ...emptyCustomer(), name };
}

function fakeApi(initial: Customer[] = []): CustomersApi {
  let rows = [...initial];
  let nextId = Math.max(0, ...rows.map(row => row.id)) + 1;
  return {
    listCustomers: vi.fn(async query => rows.filter(row => [row.name, row.tax_id, row.city].join("\n").toLowerCase().includes(query.trim().toLowerCase()))),
    createCustomer: vi.fn(async (input: CustomerInput) => { const saved = { id: nextId++, ...input }; rows = [...rows, saved]; return saved; }),
    updateCustomer: vi.fn(async (id: number, input: CustomerInput) => { const saved = { id, ...input }; rows = rows.map(row => row.id === id ? saved : row); return saved; }),
    deleteCustomer: vi.fn(async (id: number) => { rows = rows.filter(row => row.id !== id); }),
  };
}

it("creates a customer with only a name and displays saved details", async () => {
  const api = fakeApi();
  const user = userEvent.setup();
  render(<CustomersPage api={api} />);
  expect(await screen.findByText("Kartoteka jest pusta. Dodaj pierwszego kontrahenta.")).toBeVisible();
  await user.click(screen.getByRole("button", { name: "Dodaj kontrahenta" }));
  await user.click(screen.getByRole("button", { name: "Zapisz kontrahenta" }));
  expect(screen.getByText("Podaj nazwę kontrahenta.")).toBeVisible();
  expect(api.createCustomer).not.toHaveBeenCalled();
  expect(screen.getByLabelText("Kraj")).toHaveValue("");
  await user.type(screen.getByLabelText("Nazwa kontrahenta (wymagane)"), "TEST — Zażółć gęślą jaźń");
  await user.click(screen.getByRole("button", { name: "Zapisz kontrahenta" }));
  expect(await screen.findByText("Dodano kontrahenta.")).toBeVisible();
  expect(api.createCustomer).toHaveBeenCalledWith({ ...emptyCustomer(), name: "TEST — Zażółć gęślą jaźń" });
  expect(screen.getByRole("heading", { name: "TEST — Zażółć gęślą jaźń" })).toBeVisible();
  expect(screen.getByRole("button", { name: "Edytuj kontrahenta" })).toBeEnabled();
});

it("searches by the supplied query and distinguishes no matches from a load failure", async () => {
  const api = fakeApi([customer(), { ...customer(2, "Test Łódź"), tax_id: "ID-TEST", city: "Łódź" }]);
  const user = userEvent.setup();
  vi.mocked(api.listCustomers).mockRejectedValueOnce({ message: "Testowy błąd odczytu" });
  render(<CustomersPage api={api} />);
  expect(await screen.findByRole("alert")).toHaveTextContent("Testowy błąd odczytu");
  await user.click(screen.getByRole("button", { name: "Spróbuj ponownie" }));
  expect(await screen.findByRole("button", { name: /^Kontrahent testowy/ })).toBeVisible();
  await user.type(screen.getByLabelText("Szukaj kontrahenta"), "ŁÓDŹ");
  expect(await screen.findByRole("button", { name: /^Test Łódź/ })).toBeVisible();
  await waitFor(() => expect(screen.queryByRole("button", { name: /^Kontrahent testowy/ })).not.toBeInTheDocument());
  expect(api.listCustomers).toHaveBeenLastCalledWith("ŁÓDŹ");
  await user.clear(screen.getByLabelText("Szukaj kontrahenta"));
  await user.type(screen.getByLabelText("Szukaj kontrahenta"), "nieobecny");
  expect(await screen.findByText("Brak kontrahentów pasujących do wyszukiwania.")).toBeVisible();
});

it("retains the edit draft on backend validation failure and updates the same customer after retry", async () => {
  const api = fakeApi([customer()]);
  const update = api.updateCustomer;
  api.updateCustomer = vi.fn().mockRejectedValueOnce({ message: "Sprawdź dane kontrahenta.", field_errors: { email: "Testowy błąd e-mail" } }).mockImplementation(update);
  const user = userEvent.setup();
  render(<CustomersPage api={api} />);
  await user.click(await screen.findByRole("button", { name: /^Kontrahent testowy/ }));
  await user.click(screen.getByRole("button", { name: "Edytuj kontrahenta" }));
  await user.clear(screen.getByLabelText("Nazwa kontrahenta (wymagane)"));
  await user.type(screen.getByLabelText("Nazwa kontrahenta (wymagane)"), "Zmieniony test");
  await user.type(screen.getByLabelText("E-mail"), "bad");
  await user.click(screen.getByRole("button", { name: "Zapisz kontrahenta" }));
  expect(await screen.findByText("Testowy błąd e-mail")).toBeVisible();
  expect(screen.getByLabelText("E-mail")).toHaveAttribute("aria-invalid", "true");
  expect(screen.getByLabelText("Nazwa kontrahenta (wymagane)")).toHaveValue("Zmieniony test");
  expect(screen.queryByText("Zapisano zmiany kontrahenta.")).not.toBeInTheDocument();
  await user.clear(screen.getByLabelText("E-mail"));
  await user.type(screen.getByLabelText("E-mail"), "test@example.test");
  await user.click(screen.getByRole("button", { name: "Zapisz kontrahenta" }));
  expect(await screen.findByText("Zapisano zmiany kontrahenta.")).toBeVisible();
  expect(api.updateCustomer).toHaveBeenLastCalledWith(1, { ...emptyCustomer(), name: "Zmieniony test", email: "test@example.test" });
  expect(api.createCustomer).not.toHaveBeenCalled();
});

it("keeps unsaved changes when adding again or declining cancellation and row selection", async () => {
  const api = fakeApi([customer()]);
  const dirty = vi.fn();
  const user = userEvent.setup();
  render(<CustomersPage api={api} onDirtyChanged={dirty} />);
  const existing = await screen.findByRole("button", { name: /^Kontrahent testowy/ });
  await user.click(screen.getByRole("button", { name: "Dodaj kontrahenta" }));
  await user.type(screen.getByLabelText("Nazwa kontrahenta (wymagane)"), "Niezapisany test");
  await user.click(screen.getByRole("button", { name: "Dodaj kontrahenta" }));
  await user.click(screen.getByRole("button", { name: "Anuluj" }));
  let dialog = screen.getByRole("dialog", { name: "Niezapisane zmiany kontrahenta" });
  await user.click(within(dialog).getByRole("button", { name: "Anuluj" }));
  await user.click(existing);
  dialog = screen.getByRole("dialog", { name: "Niezapisane zmiany kontrahenta" });
  await user.click(within(dialog).getByRole("button", { name: "Anuluj" }));
  expect(screen.getByLabelText("Nazwa kontrahenta (wymagane)")).toHaveValue("Niezapisany test");
  expect(dirty).toHaveBeenLastCalledWith(true);
  await user.click(screen.getByRole("button", { name: "Anuluj" }));
  dialog = screen.getByRole("dialog", { name: "Niezapisane zmiany kontrahenta" });
  await user.click(within(dialog).getByRole("button", { name: "Odrzuć zmiany" }));
  expect(screen.queryByRole("heading", { name: "Nowy kontrahent" })).not.toBeInTheDocument();
  expect(dirty).toHaveBeenLastCalledWith(false);
});

it("requires confirmation for deletion and preserves the selected record when it is linked", async () => {
  const api = fakeApi([customer()]);
  const user = userEvent.setup();
  render(<CustomersPage api={api} />);
  await user.click(await screen.findByRole("button", { name: /^Kontrahent testowy/ }));
  await user.click(screen.getByRole("button", { name: "Usuń kontrahenta" }));
  let dialog = screen.getByRole("dialog", { name: "Usunięcie kontrahenta" });
  expect(within(dialog).getByText("Kontrahent testowy")).toBeVisible();
  expect(api.deleteCustomer).not.toHaveBeenCalled();
  await user.click(within(dialog).getByRole("button", { name: "Anuluj" }));
  vi.mocked(api.deleteCustomer).mockRejectedValueOnce({ code: "customer_in_use", message: "Nie można usunąć kontrahenta powiązanego z dokumentem." });
  await user.click(screen.getByRole("button", { name: "Usuń kontrahenta" }));
  dialog = screen.getByRole("dialog", { name: "Usunięcie kontrahenta" });
  await user.click(within(dialog).getByRole("button", { name: "Usuń kontrahenta" }));
  expect(await within(dialog).findByRole("alert")).toHaveTextContent("powiązanego z dokumentem");
  expect(screen.getByRole("heading", { name: "Kontrahent testowy" })).toBeVisible();
  await user.click(within(dialog).getByRole("button", { name: "Usuń kontrahenta" }));
  expect(await screen.findByText("Usunięto kontrahenta.")).toBeVisible();
  expect(await screen.findByText("Kartoteka jest pusta. Dodaj pierwszego kontrahenta.")).toBeVisible();
});

it("ignores a late search result after a newer query has completed", async () => {
  const api = fakeApi();
  let resolveOld: (rows: Customer[]) => void = () => {};
  vi.mocked(api.listCustomers).mockImplementation(query => query === "stary"
    ? new Promise(resolve => { resolveOld = resolve; })
    : Promise.resolve(query === "nowy" ? [customer(2, "Nowy test")] : []));
  const user = userEvent.setup();
  render(<CustomersPage api={api} />);
  await screen.findByText("Kartoteka jest pusta. Dodaj pierwszego kontrahenta.");
  await user.type(screen.getByLabelText("Szukaj kontrahenta"), "stary");
  await waitFor(() => expect(api.listCustomers).toHaveBeenCalledWith("stary"));
  await user.clear(screen.getByLabelText("Szukaj kontrahenta"));
  await user.type(screen.getByLabelText("Szukaj kontrahenta"), "nowy");
  expect(await screen.findByRole("button", { name: /^Nowy test/ })).toBeVisible();
  resolveOld([customer(1, "Stary test")]);
  await waitFor(() => expect(screen.queryByRole("button", { name: /^Stary test/ })).not.toBeInTheDocument());
  expect(within(screen.getByRole("region", { name: /Kartoteka/ })).getByRole("button", { name: /^Nowy test/ })).toBeVisible();
});
