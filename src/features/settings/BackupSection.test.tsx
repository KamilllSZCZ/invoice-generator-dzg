import { render,screen,waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect,it,vi } from "vitest";
import { BackupSection } from "./BackupSection";
import { backup,fakeApi } from "./testFixtures";
it("uses selected destinations, displays the actual suffixed path and treats cancellation as no-op",async()=>{
  const api=fakeApi();api.chooseBackupDestination=vi.fn().mockResolvedValueOnce(null).mockResolvedValueOnce("C:/one/copy.zip").mockResolvedValueOnce("D:/two/copy.zip");api.createBackup=vi.fn().mockResolvedValue(backup);
  const u=userEvent.setup();render(<BackupSection api={api} onRestored={async()=>{}}/>);await u.click(screen.getByRole("button",{name:"Utwórz kopię"}));expect(api.createBackup).not.toHaveBeenCalled();
  await u.click(screen.getByRole("button",{name:"Utwórz kopię"}));expect(await screen.findByRole("status")).toHaveTextContent(backup.path);expect(api.createBackup).toHaveBeenLastCalledWith("C:/one/copy.zip");
  await u.click(screen.getByRole("button",{name:"Utwórz kopię"}));await waitFor(()=>expect(api.createBackup).toHaveBeenLastCalledWith("D:/two/copy.zip"));
});
it("does not offer confirmation for an invalid archive or restore on dialog cancellation",async()=>{
  const api=fakeApi();api.chooseBackupSource=vi.fn().mockResolvedValueOnce(null).mockResolvedValue("bad.zip");api.inspectBackup=async()=>{throw new Error("Uszkodzona kopia");};api.restoreBackup=vi.fn();
  const u=userEvent.setup();render(<BackupSection api={api} onRestored={async()=>{}}/>);await u.click(screen.getByRole("button",{name:"Wybierz kopię do przywrócenia"}));expect(screen.queryByRole("dialog")).not.toBeInTheDocument();await u.click(screen.getByRole("button",{name:"Wybierz kopię do przywrócenia"}));expect(await screen.findByRole("alert")).toHaveTextContent("Uszkodzona kopia");expect(api.restoreBackup).not.toHaveBeenCalled();
});
it("shows a validated summary and requires a second explicit replace confirmation",async()=>{
  const api=fakeApi();api.chooseBackupSource=async()=>"copy.zip";api.restoreBackup=vi.fn().mockResolvedValue({restored_schema_version:2,recovery_backup_path:"C:/recovery.zip"});const refreshed=vi.fn();
  const u=userEvent.setup();render(<BackupSection api={api} onRestored={refreshed}/>);await u.click(screen.getByRole("button",{name:"Wybierz kopię do przywrócenia"}));expect(await screen.findByRole("dialog")).toHaveTextContent("2026-11-13T18:00:00Z");expect(api.restoreBackup).not.toHaveBeenCalled();
  const replace=screen.getByRole("button",{name:"Zastąp wszystkie lokalne dane"});expect(replace).toBeDisabled();await u.click(screen.getByLabelText("Rozumiem, że wszystkie lokalne dane zostaną zastąpione"));await u.click(replace);expect(await screen.findByRole("status")).toHaveTextContent("C:/recovery.zip");expect(api.restoreBackup).toHaveBeenCalledWith("copy.zip",true,backup.fingerprint);expect(refreshed).toHaveBeenCalledOnce();
});
it("keeps a failed restore out of the success path and can cancel confirmation",async()=>{
  const api=fakeApi();api.chooseBackupSource=async()=>"copy.zip";api.restoreBackup=async()=>{throw new Error("Nie przywrócono");};const refreshed=vi.fn();const u=userEvent.setup();render(<BackupSection api={api} onRestored={refreshed}/>);
  await u.click(screen.getByRole("button",{name:"Wybierz kopię do przywrócenia"}));await screen.findByRole("dialog");await u.click(screen.getByRole("button",{name:"Anuluj przywracanie"}));expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  await u.click(screen.getByRole("button",{name:"Wybierz kopię do przywrócenia"}));await u.click(await screen.findByLabelText("Rozumiem, że wszystkie lokalne dane zostaną zastąpione"));await u.click(screen.getByRole("button",{name:"Zastąp wszystkie lokalne dane"}));expect(await screen.findByRole("alert")).toHaveTextContent("Nie przywrócono");expect(refreshed).not.toHaveBeenCalled();expect(screen.queryByRole("status")).not.toBeInTheDocument();
});
