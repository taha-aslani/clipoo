import { Search } from "lucide-react";
import { forwardRef } from "react";
import { Input } from "@/components/ui/input";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
}

export const SearchBar = forwardRef<HTMLInputElement, SearchBarProps>(
  function SearchBar({ value, onChange }, ref) {
    return (
      <div className="sticky top-0 z-10 border-b border-white/5 bg-[#0F1115]/80 px-4 py-3 backdrop-blur-xl">
        <label className="sr-only" htmlFor="clipoo-search">
          جستجو
        </label>
        <div className="relative">
          <Search className="pointer-events-none absolute top-1/2 right-3 size-4 -translate-y-1/2 text-[#9CA3AF]" />
          <Input
            ref={ref}
            id="clipoo-search"
            dir="rtl"
            value={value}
            onChange={(event) => onChange(event.currentTarget.value)}
            placeholder="جستجو در کلیپ‌بورد..."
            className="h-11 rounded-[16px] border-white/10 bg-[#181C23] px-4 ps-10 text-sm text-[#F3F4F6] shadow-none transition-[border-color,box-shadow] duration-[180ms] placeholder:text-[#9CA3AF] focus-visible:border-[#22C55E]/60 focus-visible:ring-[#22C55E]/20"
          />
        </div>
      </div>
    );
  },
);
