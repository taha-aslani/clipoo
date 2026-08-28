import { useCallback, useEffect, useState } from "react";
import {
  copyClipboardItem,
  deleteClipboardItem,
  listClipboardItems,
  pinClipboardItem,
  searchClipboardItems,
} from "@/lib/api/clipboard";
import { CLIPBOARD_ITEM_ADDED_EVENT } from "@/lib/clipboard-events";
import { isTauriRuntime } from "@/lib/runtime";
import { hideMainWindow } from "@/lib/window";
import type { ClipboardFilter, ClipboardItem } from "@/types/clipboard";

export function useClipboardItems(filter: ClipboardFilter, query: string) {
  const [items, setItems] = useState<ClipboardItem[]>([]);

  const copyItem = useCallback(async (id: string) => {
    try {
      await copyClipboardItem(id);
      await hideMainWindow();
    } catch {
      // Keep the window open when native copy fails.
    }
  }, []);

  const pinItem = useCallback(
    async (id: string, pinned: boolean) => {
      try {
        const updated = await pinClipboardItem(id, pinned);
        setItems((current) => {
          if (filter === "pinned" && !updated.pinned) {
            return current.filter((item) => item.id !== updated.id);
          }
          return current.map((item) => (item.id === updated.id ? updated : item));
        });
      } catch {
        // Leave the current list unchanged.
      }
    },
    [filter],
  );

  const deleteItem = useCallback(async (id: string) => {
    try {
      await deleteClipboardItem(id);
      setItems((current) => current.filter((item) => item.id !== id));
    } catch {
      // Leave the current list unchanged.
    }
  }, []);

  const reload = useCallback(async () => {
    if (!isTauriRuntime()) {
      return;
    }

    try {
      const result = query.trim()
        ? await searchClipboardItems(query, filter)
        : await listClipboardItems(filter);
      setItems(result);
    } catch {
      setItems([]);
    }
  }, [filter, query]);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }

    let cancelled = false;

    const load = async () => {
      try {
        const result = query.trim()
          ? await searchClipboardItems(query, filter)
          : await listClipboardItems(filter);
        if (!cancelled) {
          setItems(result);
        }
      } catch {
        if (!cancelled) {
          setItems([]);
        }
      }
    };

    void load();

    let unlisten: (() => void) | undefined;
    void import("@tauri-apps/api/event")
      .then(({ listen }) =>
        listen<ClipboardItem>(CLIPBOARD_ITEM_ADDED_EVENT, (event) => {
          if (query.trim()) {
            void load();
            return;
          }

          const item = event.payload;
          const matchesFilter =
            filter === "all" ||
            (filter === "pinned" && item.pinned) ||
            item.type === filter;

          if (!matchesFilter) {
            return;
          }

          setItems((current) => [
            item,
            ...current.filter((existing) => existing.id !== item.id),
          ]);
        }),
      )
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {
        unlisten = undefined;
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [filter, query]);

  return { items, reload, copyItem, pinItem, deleteItem };
}
