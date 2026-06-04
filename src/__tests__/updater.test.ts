/**
 * @vitest-environment jsdom
 */
import { beforeEach, describe, expect, it, vi } from "vitest";

describe("app updater", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.unstubAllGlobals();
  });

  it("skips update checks outside the Tauri runtime", async () => {
    const notify = vi.fn();
    const { checkForAppUpdate } = await import("../api/updater");

    const result = await checkForAppUpdate({ notify });

    expect(result.status).toBe("unavailable");
    expect(notify).not.toHaveBeenCalled();
  });

  it("reports when no GitHub release update is available", async () => {
    vi.stubGlobal("__TAURI_INTERNALS__", {});
    vi.doMock("@tauri-apps/plugin-updater", () => ({
      check: vi.fn().mockResolvedValue(null),
    }));

    const { checkForAppUpdate } = await import("../api/updater");
    const result = await checkForAppUpdate({ notify: vi.fn() });

    expect(result.status).toBe("idle");
  });

  it("downloads, installs, and restarts when the user accepts an update", async () => {
    vi.stubGlobal("__TAURI_INTERNALS__", {});
    const downloadAndInstall = vi.fn().mockResolvedValue(undefined);
    const relaunch = vi.fn().mockResolvedValue(undefined);

    vi.doMock("@tauri-apps/plugin-updater", () => ({
      check: vi.fn().mockResolvedValue({
        version: "0.2.0",
        downloadAndInstall,
      }),
    }));
    vi.doMock("@tauri-apps/plugin-process", () => ({
      relaunch,
    }));

    const notify = vi.fn();
    const confirmInstall = vi.fn().mockResolvedValue(true);
    const { checkForAppUpdate } = await import("../api/updater");

    const result = await checkForAppUpdate({ notify, confirmInstall });

    expect(confirmInstall).toHaveBeenCalledWith("0.2.0");
    expect(downloadAndInstall).toHaveBeenCalledOnce();
    expect(relaunch).toHaveBeenCalledOnce();
    expect(notify).toHaveBeenCalledWith("更新已安装，正在重启应用。", "success");
    expect(result.status).toBe("installed");
  });

  it("notifies when the update check fails", async () => {
    vi.stubGlobal("__TAURI_INTERNALS__", {});
    vi.doMock("@tauri-apps/plugin-updater", () => ({
      check: vi.fn().mockRejectedValue(new Error("network failed")),
    }));

    const notify = vi.fn();
    const { checkForAppUpdate } = await import("../api/updater");

    const result = await checkForAppUpdate({ notify });

    expect(result.status).toBe("error");
    expect(notify).toHaveBeenCalledWith("检查更新失败：network failed", "warning");
  });
});
