import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauriRuntime } from "@/lib/runtime";

export async function hideMainWindow(): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  await getCurrentWindow().hide();
}
