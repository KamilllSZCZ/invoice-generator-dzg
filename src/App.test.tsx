import { render,screen,within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { it,expect,vi } from "vitest";
import App from "./App";
vi.mock("./features/settings/api",async original=>{const actual=await original<typeof import("./features/settings/api")>();const {fakeApi}=await import("./features/settings/testFixtures");return {...actual,settingsApi:fakeApi()};});
vi.mock("./features/customers/api", async original => {
  const actual = await original<typeof import("./features/customers/api")>();
  return { ...actual, customersApi: { listCustomers: vi.fn(async () => []), createCustomer: vi.fn(), updateCustomer: vi.fn(), deleteCustomer: vi.fn() } };
});
it("keeps unsaved-change protection when clicking the current sidebar route",async()=>{
  const u=userEvent.setup();render(<App/>);
  await u.click(screen.getByRole("button",{name:"Ustawienia"}));await u.type(await screen.findByLabelText("Nazwa sprzedawcy"),"Szkic");await u.click(screen.getByRole("button",{name:"Ustawienia"}));await u.click(screen.getByRole("button",{name:"Pulpit"}));
  const dialog=screen.getByRole("dialog",{name:"Niezapisane zmiany ustawień"});await u.click(within(dialog).getByRole("button",{name:"Anuluj"}));expect(screen.getByLabelText("Nazwa sprzedawcy")).toHaveValue("Szkic");
});
it("protects a customer draft when switching sidebar routes", async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole("button", { name: "Kontrahenci" }));
  await user.click(await screen.findByRole("button", { name: "Dodaj kontrahenta" }));
  await user.type(screen.getByLabelText("Nazwa kontrahenta (wymagane)"), "Szkic kontrahenta");
  await user.click(screen.getByRole("button", { name: "Kontrahenci" }));
  await user.click(screen.getByRole("button", { name: "Pulpit" }));
  let dialog = screen.getByRole("dialog", { name: "Niezapisane zmiany kontrahenta" });
  await user.click(within(dialog).getByRole("button", { name: "Anuluj" }));
  expect(screen.getByLabelText("Nazwa kontrahenta (wymagane)")).toHaveValue("Szkic kontrahenta");
  await user.click(screen.getByRole("button", { name: "Pulpit" }));
  dialog = screen.getByRole("dialog", { name: "Niezapisane zmiany kontrahenta" });
  await user.click(within(dialog).getByRole("button", { name: "Odrzuć zmiany" }));
  expect(screen.getByRole("heading", { name: "Pulpit" })).toBeVisible();
});
