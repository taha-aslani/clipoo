import { useVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useRef } from "react";
import { ClipboardCard } from "@/features/clipboard/components/ClipboardCard";
import type { ClipboardItem } from "@/types/clipboard";

interface ClipboardListProps {
  items: ClipboardItem[];
  query: string;
  selectedIndex: number;
  onSelectIndex: (index: number) => void;
  onCopy: (id: string) => void;
  onPin: (id: string, pinned: boolean) => void;
  onDelete: (id: string) => void;
}

export function ClipboardList({
  items,
  query,
  selectedIndex,
  onSelectIndex,
  onCopy,
  onPin,
  onDelete,
}: ClipboardListProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 108,
    overscan: 10,
    getItemKey: (index) => items[index]?.id ?? index,
  });

  useEffect(() => {
    if (items.length === 0) {
      return;
    }

    virtualizer.scrollToIndex(selectedIndex, { align: "auto" });
  }, [selectedIndex, items.length, virtualizer]);

  if (items.length === 0) {
    const searching = query.trim().length > 0;

    return (
      <section className="flex flex-1 items-center justify-center p-6">
        <div className="rounded-[16px] border border-white/10 bg-[#181C23]/70 px-8 py-10 text-center backdrop-blur-xl">
          <p className="text-base font-medium">
            {searching ? "نتیجه‌ای پیدا نشد" : "تاریخچه خالی است"}
          </p>
          <p className="mt-2 text-sm text-[#9CA3AF]">
            {searching
              ? "عبارت دیگری را امتحان کنید."
              : "هر متن، تصویر، فایل یا لینکی که کپی کنید اینجا ذخیره می‌شود."}
          </p>
        </div>
      </section>
    );
  }

  return (
    <section ref={parentRef} className="flex-1 overflow-y-auto p-4">
      <div
        role="listbox"
        aria-label="تاریخچه کلیپ‌بورد"
        className="relative w-full"
        style={{ height: virtualizer.getTotalSize() }}
      >
        {virtualizer.getVirtualItems().map((row) => {
          const item = items[row.index];
          if (!item) {
            return null;
          }

          return (
            <div
              key={row.key}
              data-index={row.index}
              ref={virtualizer.measureElement}
              className="absolute top-0 right-0 w-full pb-2"
              style={{ transform: `translateY(${row.start}px)` }}
            >
              <ClipboardCard
                item={item}
                query={query}
                selected={row.index === selectedIndex}
                onSelect={() => onSelectIndex(row.index)}
                onCopy={() => onCopy(item.id)}
                onPin={() => onPin(item.id, !item.pinned)}
                onDelete={() => onDelete(item.id)}
              />
            </div>
          );
        })}
      </div>
    </section>
  );
}
