import { render,screen,waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe,it,expect,vi } from "vitest";
import { SettingsPage } from "./SettingsPage";
import { fakeApi } from "./testFixtures";

describe("panel ustawień",()=>{
  it("loads once, saves optional seller data and reloads persisted values",async()=>{
    const api=fakeApi();const user=userEvent.setup();const view=render(<SettingsPage api={api}/>);
    expect(screen.getByRole("status")).toHaveTextContent(/Wczytywanie/);
    const name=await screen.findByLabelText("Nazwa sprzedawcy");await user.type(name,"Moja firma");
    expect(screen.getByText("Niezapisane zmiany")).toBeInTheDocument();
    await user.click(screen.getByRole("button",{name:"Zapisz dane sprzedawcy"}));
    expect(await screen.findByText("Zapisano zmiany.")).toBeInTheDocument();view.unmount();render(<SettingsPage api={api}/>);
    expect(await screen.findByLabelText("Nazwa sprzedawcy")).toHaveValue("Moja firma");
    expect(api.saved().company.tax_id).toBeNull();expect(api.saved().company.registration_id).toBeNull();
  });
  it("announces backend field errors and never reports a failed save as success",async()=>{
    const api=fakeApi();api.saveCompanySettings=async()=>{throw {code:"validation_error",message:"Błąd danych",field_errors:{email:"Nieprawidłowy adres"}}};
    const u=userEvent.setup();render(<SettingsPage api={api}/>);await u.type(await screen.findByLabelText("E-mail"),"a@example.pl");await u.click(screen.getByRole("button",{name:"Zapisz dane sprzedawcy"}));
    expect(await screen.findByRole("alert")).toHaveTextContent("Nieprawidłowy adres");expect(screen.queryByText("Zapisano zmiany.")).not.toBeInTheDocument();
  });
  it("preserves drafts between sections, validates defaults, and saves all document fields",async()=>{
    const api=fakeApi();const u=userEvent.setup();render(<SettingsPage api={api}/>);await u.type(await screen.findByLabelText("Nazwa sprzedawcy"),"Szkic");
    await u.click(screen.getByRole("button",{name:"Dokumenty"}));await u.type(screen.getByLabelText("Waluta"),"PLN");await u.type(screen.getByLabelText("Termin płatności (dni)"),"14");
    await u.type(screen.getByLabelText("Miejsce wystawienia"),"Łańcut");await u.type(screen.getByLabelText("Uwagi domyślne"),"Dziękujemy");
    await u.click(screen.getByRole("button",{name:"Zapisz ustawienia dokumentów"}));await waitFor(()=>expect(api.saved().document_defaults.issue_place).toBe("Łańcut"));
    await u.click(screen.getByRole("button",{name:"Dane sprzedawcy"}));expect(screen.getByLabelText("Nazwa sprzedawcy")).toHaveValue("Szkic");
  });
  it("previews saved numbering without saving or changing a draft counter",async()=>{
    const api=fakeApi();const u=userEvent.setup();render(<SettingsPage api={api}/>);await screen.findByLabelText("Nazwa sprzedawcy");await u.click(screen.getByRole("button",{name:"Numeracja"}));
    const counter=screen.getByLabelText("Następny licznik");await u.clear(counter);await u.type(counter,"8");await u.click(screen.getByRole("button",{name:"Odśwież podgląd"}));
    expect(await screen.findByText("1/2026")).toBeInTheDocument();expect(counter).toHaveValue("8");expect(api.saved().numbering[0].next_counter).toBe(1);
    await u.click(screen.getByRole("button",{name:"Zapisz numerację"}));await waitFor(()=>expect(api.saved().numbering[0].next_counter).toBe(8));
  });
  it("retains edit state and exposes retry when initial loading fails",async()=>{
    const api=fakeApi();api.getSettings=vi.fn().mockRejectedValueOnce(new Error("brak dostępu")).mockResolvedValue(api.saved());
    const u=userEvent.setup();render(<SettingsPage api={api}/>);expect(await screen.findByRole("alert")).toHaveTextContent(/brak dostępu/);await u.click(screen.getByRole("button",{name:"Spróbuj ponownie"}));expect(await screen.findByLabelText("Nazwa sprzedawcy")).toBeInTheDocument();
  });
});
