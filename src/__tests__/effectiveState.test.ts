import { describe, expect, it } from "vitest";
import { ruleLabel } from "../types";

describe("ruleLabel", () => {
  it("uses friendly project rule labels", () => {
    expect(ruleLabel(undefined)).toBe("跟随默认");
    expect(ruleLabel("inherit")).toBe("跟随默认");
    expect(ruleLabel("enable")).toBe("在此项目启用");
    expect(ruleLabel("disable")).toBe("在此项目停用");
  });
});
