import { useCallback, useEffect, useRef, useState } from "react";
import { ClipboardList } from "@/features/clipboard/components/ClipboardList";
import { Sidebar } from "@/features/clipboard/components/Sidebar";
import { useClipboardItems } from "@/features/clipboard/hooks/useClipboardItems";
import type { ClipboardFilter } from "@/features/clipboard/types";
import { useClipooKeyboard } from "@/features/keyboard/useClipooKeyboard";
import { SearchBar } from "@/features/search/components/SearchBar";
import { SEARCH_DEBOUNCE_MS } from "@/features/search/constants";
import { useDebouncedValue } from "@/features/search/hooks/useDebouncedValue";
import { SettingsPanel } from "@/features/settings/components/SettingsPanel";
import { WINDOW_SHOWN_EVENT } from "@/lib/clipboard-events";
import { isTauriRuntime } from "@/lib/runtime";
import { hideMainWindow } from "@/lib/window";

export default function App() {
  const [activeFilter, setActiveFilter] = useState<ClipboardFilter>("all");
  const [query, setQuery] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const searchRef = useRef<HTMLInputElement>(null);
  const debouncedQuery = useDebouncedValue(query, SEARCH_DEBOUNCE_MS);
  const { items, reload, copyItem, pinItem, deleteItem } = useClipboardItems(
    activeFilter,
    debouncedQuery,
  );

  useEffect(() => {
    setSelectedIndex(0);
  }, [activeFilter, debouncedQuery]);

  useEffect(() => {
    if (selectedIndex >= items.length) {
      setSelectedIndex(Math.max(0, items.length - 1));
    }
  }, [items.length, selectedIndex]);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }

    let unlisten: (() => void) | undefined;
    void import("@tauri-apps/api/event")
      .then(({ listen }) => listen(WINDOW_SHOWN_EVENT, () => {
        searchRef.current?.focus();
        setSelectedIndex(0);
      }))
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {
        unlisten = undefined;
      });

    return () => {
      unlisten?.();
    };
  }, []);

  const onCopySelected = useCallback(() => {
    const item = items[selectedIndex];
    if (!item) {
      return;
    }
    void copyItem(item.id);
  }, [copyItem, items, selectedIndex]);

  const onEscape = useCallback(() => {
    if (settingsOpen) {
      setSettingsOpen(false);
      return;
    }
    void hideMainWindow();
  }, [settingsOpen]);

  useClipooKeyboard({
    enabled: true,
    itemCount: settingsOpen ? 0 : items.length,
    selectedIndex,
    onSelectedIndexChange: setSelectedIndex,
    onCopySelected,
    onEscape,
  });

  return (
    <div className="flex h-svh overflow-hidden bg-[#0F1115] text-[#F3F4F6]">
      <Sidebar
        activeFilter={activeFilter}
        onFilterChange={(filter) => {
          setSettingsOpen(false);
          setActiveFilter(filter);
        }}
        settingsOpen={settingsOpen}
        onOpenSettings={() => setSettingsOpen(true)}
      />
      <main className="flex min-w-0 flex-1 flex-col">
        {settingsOpen ? (
          <SettingsPanel
            onBack={() => setSettingsOpen(false)}
            onHistoryCleared={() => {
              void reload();
            }}
          />
        ) : (
          <>
            <SearchBar ref={searchRef} value={query} onChange={setQuery} />
            <ClipboardList
              items={items}
              query={debouncedQuery}
              selectedIndex={selectedIndex}
              onSelectIndex={setSelectedIndex}
              onCopy={(id) => {
                void copyItem(id);
              }}
              onPin={(id, pinned) => {
                void pinItem(id, pinned);
              }}
              onDelete={(id) => {
                void deleteItem(id);
              }}
            />
          </>
        )}
      </main>
    </div>
  );
}
