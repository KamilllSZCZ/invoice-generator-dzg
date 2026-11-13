import { render,screen,waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect,it,vi } from "vitest";
import { SettingsPage } from "./SettingsPage";
import { fakeApi } from "./testFixtures";
it("applies theme and density only after successful persistence and displays real metadata",async()=>{
  const api=fakeApi();const applied=vi.fn();const u=userEvent.setup();render(<SettingsPage api={api} onPreferencesChanged={applied}/>);await screen.findByLabelText("Nazwa sprzedawcy");await u.click(screen.getByRole("button",{name:"Aplikacja"}));
  expect(await screen.findByText("0.1.0")).toBeInTheDocument();await u.selectOptions(screen.getByLabelText("Motyw"),"light");await u.selectOptions(screen.getByLabelText("Gęstość interfejsu"),"compact");
  expect(api.saved().preferences.theme).toBeNull();await u.click(screen.getByRole("button",{name:"Zapisz preferencje"}));await waitFor(()=>expect(applied).toHaveBeenLastCalledWith(expect.objectContaining({theme:"light",density:"compact"})));
  await u.selectOptions(screen.getByLabelText("Motyw"),"system");await u.click(screen.getByRole("button",{name:"Zapisz preferencje"}));await waitFor(()=>expect(api.saved().preferences.theme).toBe("system"));
});
it("never applies a failed preference update",async()=>{
  const api=fakeApi();api.saveAppPreferences=async()=>{throw new Error("Nie zapisano");};const applied=vi.fn();const u=userEvent.setup();render(<SettingsPage api={api} onPreferencesChanged={applied}/>);await screen.findByLabelText("Nazwa sprzedawcy");await u.click(screen.getByRole("button",{name:"Aplikacja"}));await u.selectOptions(screen.getByLabelText("Motyw"),"dark");await u.click(screen.getByRole("button",{name:"Zapisz preferencje"}));expect(await screen.findByRole("alert")).toHaveTextContent("Nie zapisano");expect(applied).toHaveBeenCalledTimes(1);
});
