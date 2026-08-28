import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "@/types/settings";

export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke<AppSettings>("update_settings", { settings });
}
