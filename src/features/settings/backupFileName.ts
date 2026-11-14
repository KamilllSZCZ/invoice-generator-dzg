export function timestampedBackupFileName(now:Date):string {
  const pad=(n:number)=>String(n).padStart(2,"0");
  return `dualzone-garage-backup-${now.getUTCFullYear()}${pad(now.getUTCMonth()+1)}${pad(now.getUTCDate())}-${pad(now.getUTCHours())}${pad(now.getUTCMinutes())}${pad(now.getUTCSeconds())}Z.zip`;
}
