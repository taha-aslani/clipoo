import { convertFileSrc } from "@tauri-apps/api/core";
import { isTauriRuntime } from "@/lib/runtime";

export function clipboardImageSrc(path: string): string | null {
  if (!path || !isTauriRuntime()) {
    return null;
  }

  return convertFileSrc(path);
}
