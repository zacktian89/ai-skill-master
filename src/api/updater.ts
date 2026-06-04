import { hasTauriRuntime } from "./client";

export type AppUpdateStatus = "unavailable" | "checking" | "idle" | "available" | "installing" | "installed" | "error";
export type UpdateNoticeType = "error" | "success" | "warning" | "info";

export interface AppUpdateResult {
  status: AppUpdateStatus;
  version?: string;
  error?: string;
}

export interface CheckForAppUpdateOptions {
  notify?: (message: string, type?: UpdateNoticeType) => void;
  confirmInstall?: (version: string) => boolean | Promise<boolean>;
}

function defaultConfirmInstall(version: string): boolean {
  if (typeof window === "undefined" || typeof window.confirm !== "function") return false;
  return window.confirm(`发现 SkillMaster ${version} 更新，是否立即安装？`);
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

export async function checkForAppUpdate(options: CheckForAppUpdateOptions = {}): Promise<AppUpdateResult> {
  if (!hasTauriRuntime()) {
    return { status: "unavailable" };
  }

  try {
    const [{ check }, { relaunch }] = await Promise.all([
      import("@tauri-apps/plugin-updater"),
      import("@tauri-apps/plugin-process"),
    ]);
    const update = await check();

    if (!update) {
      return { status: "idle" };
    }

    const version = update.version;
    const confirmInstall = options.confirmInstall ?? defaultConfirmInstall;
    const accepted = await confirmInstall(version);

    if (!accepted) {
      return { status: "available", version };
    }

    options.notify?.(`正在下载并安装 SkillMaster ${version}。`, "info");
    await update.downloadAndInstall();
    options.notify?.("更新已安装，正在重启应用。", "success");
    await relaunch();

    return { status: "installed", version };
  } catch (cause) {
    const message = errorMessage(cause);
    options.notify?.(`检查更新失败：${message}`, "warning");
    return { status: "error", error: message };
  }
}
