import { invoke } from "@tauri-apps/api/core";
import type { ClipboardFilter, ClipboardItem } from "@/types/clipboard";

export function listClipboardItems(
  filter: ClipboardFilter = "all",
  limit = 2000,
): Promise<ClipboardItem[]> {
  return invoke<ClipboardItem[]>("list_clipboard_items", { filter, limit });
}

export function searchClipboardItems(
  query: string,
  filter: ClipboardFilter = "all",
  limit = 2000,
): Promise<ClipboardItem[]> {
  return invoke<ClipboardItem[]>("search_clipboard_items", {
    query,
    filter,
    limit,
  });
}

export function pinClipboardItem(
  id: string,
  pinned: boolean,
): Promise<ClipboardItem> {
  return invoke<ClipboardItem>("pin_clipboard_item", { id, pinned });
}

export function deleteClipboardItem(id: string): Promise<void> {
  return invoke<void>("delete_clipboard_item", { id });
}

export function clearClipboardHistory(): Promise<number> {
  return invoke<number>("clear_clipboard_history");
}

export function copyClipboardItem(id: string): Promise<void> {
  return invoke<void>("copy_clipboard_item", { id });
}
