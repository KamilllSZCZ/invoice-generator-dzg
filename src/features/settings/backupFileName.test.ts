import { expect,it } from "vitest";
import { timestampedBackupFileName } from "./backupFileName";
it("formats UTC components including leading zeros irrespective of the local timezone",()=>{
  expect(timestampedBackupFileName(new Date("2026-01-02T03:04:05Z"))).toBe("dualzone-garage-backup-20260102-030405Z.zip");
  expect(timestampedBackupFileName(new Date("2026-11-14T00:00:00+02:00"))).toBe("dualzone-garage-backup-20261113-220000Z.zip");
});
