type DialogOpenOptions = {
  directory?: boolean;
  multiple?: boolean;
};

type DialogResult = string | string[] | null;

export async function openDirectory(options: DialogOpenOptions = {}): Promise<DialogResult> {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return "/tmp/mock-selection";
  }
  try {
    const module = await import("@tauri-apps/plugin-dialog");
    if (typeof module.open === "function") {
      return module.open(options);
    }
  } catch {
    return "/tmp/mock-selection";
  }
  return "/tmp/mock-selection";
}
