export type ClipboardItemType = "text" | "image" | "file" | "url" | "code";

export type ClipboardFilter = "all" | "text" | "image" | "file" | "url" | "pinned";

export interface ClipboardItem {
  id: string;
  type: ClipboardItemType;
  content: string;
  normalizedContent: string;
  preview: string;
  pinned: boolean;
  createdAt: number;
}
