import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("icon pipeline", () => {
  it("cache-busts the favicon asset", () => {
    const html = readFileSync(resolve(process.cwd(), "index.html"), "utf8");

    expect(html).toContain('/skillmaster-logo-dark.png?v=20260603');
  });

  it("rebuilds the desktop executable when icon assets change", () => {
    const buildScript = readFileSync(resolve(process.cwd(), "src-tauri/build.rs"), "utf8");

    expect(buildScript).toContain('println!("cargo:rerun-if-changed={path}");');
    expect(buildScript).toContain('"icons/icon.ico"');
    expect(buildScript).toContain('"icons/32x32.png"');
    expect(buildScript).toContain('"icons/128x128.png"');
    expect(buildScript).toContain('"icons/128x128@2x.png"');
  });

  it("sizes the sidebar brand logo to fill its container", () => {
    const layoutCss = readFileSync(resolve(process.cwd(), "src/styles/layout.css"), "utf8");
    const match = layoutCss.match(/\.rail-brand-mark img\s*\{([^}]*)\}/);

    expect(match?.[1]).toContain("width: 100%;");
    expect(match?.[1]).toContain("height: 100%;");
  });
});
