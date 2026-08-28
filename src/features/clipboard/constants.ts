import { FileText, Files, Image, Layers, Link2, Pin } from "lucide-react";
import type { ClipboardFilter } from "@/features/clipboard/types";

export const SIDEBAR_FILTERS: readonly {
  id: ClipboardFilter;
  label: string;
  icon: typeof Layers;
}[] = [
  { id: "all", label: "همه", icon: Layers },
  { id: "text", label: "متن", icon: FileText },
  { id: "image", label: "تصاویر", icon: Image },
  { id: "file", label: "فایل‌ها", icon: Files },
  { id: "url", label: "لینک‌ها", icon: Link2 },
  { id: "pinned", label: "سنجاق‌شده", icon: Pin },
] as const;
