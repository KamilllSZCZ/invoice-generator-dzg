import { expect,it } from "vitest";
import { validateCompany,validateDefaults,validateNumbering } from "./validation";
import { initialSettings } from "./testFixtures";
it("allows empty seller fields but rejects an invalid supplied email",()=>{expect(validateCompany(initialSettings().company)).toEqual({});expect(validateCompany({...initialSettings().company,email:"broken"})).toHaveProperty("email");});
it("rejects incompatible resets, malformed tokens and unsafe integer counters",()=>{
  for(const format of ["{counter}","{counter}/{unknown}","{counter", "x}"]){expect(validateNumbering({...initialSettings().numbering[0],reset_policy:"monthly",format})).toHaveProperty("format");}
  for(const next_counter of [0,1.5,Number.MAX_SAFE_INTEGER+1])expect(validateNumbering({...initialSettings().numbering[0],next_counter})).toHaveProperty("next_counter");
});
it("validates optional currency and payment days",()=>{expect(validateDefaults({...initialSettings().document_defaults,currency:"PL",payment_term_days:-1})).toMatchObject({currency:expect.any(String),payment_term_days:expect.any(String)});});
