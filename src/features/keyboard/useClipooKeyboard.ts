import { useEffect } from "react";

interface UseClipooKeyboardOptions {
  enabled: boolean;
  itemCount: number;
  selectedIndex: number;
  onSelectedIndexChange: (index: number) => void;
  onCopySelected: () => void;
  onEscape: () => void;
}

export function useClipooKeyboard({
  enabled,
  itemCount,
  selectedIndex,
  onSelectedIndexChange,
  onCopySelected,
  onEscape,
}: UseClipooKeyboardOptions) {
  useEffect(() => {
    if (!enabled) {
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.isComposing) {
        return;
      }

      if (event.key === "Escape") {
        event.preventDefault();
        onEscape();
        return;
      }

      if (itemCount === 0) {
        return;
      }

      if (event.key === "ArrowDown") {
        event.preventDefault();
        onSelectedIndexChange(Math.min(itemCount - 1, selectedIndex + 1));
        return;
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        onSelectedIndexChange(Math.max(0, selectedIndex - 1));
        return;
      }

      if (event.key === "Enter" && !event.shiftKey) {
        event.preventDefault();
        onCopySelected();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [
    enabled,
    itemCount,
    onCopySelected,
    onEscape,
    onSelectedIndexChange,
    selectedIndex,
  ]);
}
