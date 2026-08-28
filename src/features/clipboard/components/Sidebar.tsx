import { Settings } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { ClipboardFilter } from "@/features/clipboard/types";
import { SIDEBAR_FILTERS } from "@/features/clipboard/constants";
import { cn } from "@/lib/utils";

interface SidebarProps {
  activeFilter: ClipboardFilter;
  onFilterChange: (filter: ClipboardFilter) => void;
  settingsOpen: boolean;
  onOpenSettings: () => void;
}

export function Sidebar({
  activeFilter,
  onFilterChange,
  settingsOpen,
  onOpenSettings,
}: SidebarProps) {
  return (
    <aside className="flex w-[220px] shrink-0 flex-col border-l border-white/5 bg-[#181C23]/80 backdrop-blur-xl">
      <div className="px-5 py-5">
        <p className="text-lg font-semibold tracking-tight text-[#F3F4F6]">Clipoo</p>
        <p className="mt-1 text-xs text-[#9CA3AF]">مدیریت کلیپ‌بورد</p>
      </div>
      <nav className="flex flex-col gap-1 px-3">
        {SIDEBAR_FILTERS.map((item) => {
          const Icon = item.icon;
          const isActive = item.id === activeFilter;

          return (
            <Button
              key={item.id}
              type="button"
              variant="ghost"
              onClick={() => onFilterChange(item.id)}
              className={cn(
                "h-auto w-full justify-start gap-3 rounded-[16px] px-3 py-2.5 text-sm font-normal",
                isActive
                  ? "bg-[#22C55E]/15 text-[#22C55E] hover:bg-[#22C55E]/20 hover:text-[#22C55E]"
                  : "text-[#9CA3AF] hover:bg-white/5 hover:text-[#F3F4F6]",
              )}
            >
              <Icon className="size-4 shrink-0" />
              <span>{item.label}</span>
            </Button>
          );
        })}
      </nav>
      <div className="mt-auto p-3">
        <Button
          type="button"
          variant="ghost"
          onClick={onOpenSettings}
          className={cn(
            "h-auto w-full justify-start gap-3 rounded-[16px] px-3 py-2.5 text-sm font-normal",
            settingsOpen
              ? "bg-[#22C55E]/15 text-[#22C55E] hover:bg-[#22C55E]/20 hover:text-[#22C55E]"
              : "text-[#9CA3AF] hover:bg-white/5 hover:text-[#F3F4F6]",
          )}
        >
          <Settings className="size-4 shrink-0" />
          <span>تنظیمات</span>
        </Button>
      </div>
    </aside>
  );
}
