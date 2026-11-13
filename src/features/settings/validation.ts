import type { SellerProfile,DocumentDefaults,NumberingSeriesSettings,FieldErrors } from "./types";
export const optional=(value:string)=>value===""?null:value;
export function validateCompany(s:SellerProfile):FieldErrors {
  const e:FieldErrors={};if(s.email&&!/^[^\s@]+@[^\s@.]+(?:\.[^\s@.]+)+$/.test(s.email))e.email="Podaj prawidłowy adres e-mail.";
  if(s.website&&!/^https?:\/\/[^\s/]+(?:\/[^\s]*)?$/.test(s.website))e.website="Podaj adres z http:// lub https://.";return e;
}
export function validateDefaults(s:DocumentDefaults):FieldErrors {
  const e:FieldErrors={};if(s.currency&&!/^[A-Z]{3}$/.test(s.currency))e.currency="Podaj trzyliterowy kod waluty, np. PLN.";
  if(s.payment_term_days!==null&&(!Number.isInteger(s.payment_term_days)||s.payment_term_days<0||s.payment_term_days>3650))e.payment_term_days="Podaj liczbę dni od 0 do 3650.";return e;
}
export function validateNumbering(s:NumberingSeriesSettings):FieldErrors {
  const e:FieldErrors={};const tokens=[...s.format.matchAll(/\{([^{}]*)\}/g)].map(m=>m[1]);
  if(!s.format||s.format.length>200||s.format.replace(/\{(?:prefix|year|month|counter)\}/g,"").match(/[{}\x00-\x1f]/)||!tokens.includes("counter")||(s.reset_policy!=="never"&&!tokens.includes("year"))||(s.reset_policy==="monthly"&&!tokens.includes("month")))e.format="Sprawdź tokeny. Reset roczny wymaga {year}, miesięczny także {month}. Wymagany jest {counter}.";
  if(s.prefix.length>50||/[{}\x00-\x1f]/.test(s.prefix))e.prefix="Prefiks do 50 znaków, bez nawiasów i znaków sterujących.";
  if(!Number.isSafeInteger(s.next_counter)||s.next_counter<1)e.next_counter="Podaj dodatnią, bezpieczną liczbę całkowitą.";return e;
}
