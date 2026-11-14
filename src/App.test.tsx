import { render,screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { it,expect,vi } from "vitest";
import App from "./App";
vi.mock("./features/settings/api",async original=>{const actual=await original<typeof import("./features/settings/api")>();const {fakeApi}=await import("./features/settings/testFixtures");return {...actual,settingsApi:fakeApi()};});
it("keeps unsaved-change protection when clicking the current sidebar route",async()=>{
  const confirm=vi.spyOn(window,"confirm").mockReturnValue(false);const u=userEvent.setup();render(<App/>);
  await u.click(screen.getByRole("button",{name:"Ustawienia"}));await u.type(await screen.findByLabelText("Nazwa sprzedawcy"),"Szkic");await u.click(screen.getByRole("button",{name:"Ustawienia"}));await u.click(screen.getByRole("button",{name:"Pulpit"}));
  expect(confirm).toHaveBeenCalledOnce();expect(screen.getByLabelText("Nazwa sprzedawcy")).toHaveValue("Szkic");confirm.mockRestore();
});
