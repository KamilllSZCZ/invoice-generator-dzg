import { render,screen,waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect,it } from "vitest";
import { SettingsPage } from "./SettingsPage";
import { fakeApi } from "./testFixtures";
it("previews draft footer, variant and visibility before explicitly saving",async()=>{
  const api=fakeApi();const u=userEvent.setup();render(<SettingsPage api={api}/>);await screen.findByLabelText("Nazwa sprzedawcy");await u.click(screen.getByRole("button",{name:"Wygląd dokumentów"}));
  await u.type(screen.getByLabelText("Stopka dokumentu"),"Dziękujemy za wizytę");await u.selectOptions(screen.getByLabelText("Wariant dokumentu"),"modern");await u.click(screen.getByLabelText("Pokaż e-mail"));
  expect(screen.getByLabelText("Przykładowy dokument")).toHaveTextContent("Dziękujemy za wizytę");expect(screen.getByLabelText("Przykładowy dokument")).not.toHaveTextContent("biuro@example.test");expect(api.saved().appearance.footer).toBeNull();
  await u.click(screen.getByRole("button",{name:"Zapisz wygląd dokumentów"}));await waitFor(()=>expect(api.saved().appearance).toMatchObject({footer:"Dziękujemy za wizytę",template:"modern",show_email:false}));
});
