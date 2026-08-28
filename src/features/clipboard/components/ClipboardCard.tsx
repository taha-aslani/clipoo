import { Code2, Copy, FileText, Files, Image, Link2, Pin, Trash2 } from "lucide-react";
import { useState, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { clipboardImageSrc } from "@/features/clipboard/image-src";
import { HighlightedText } from "@/features/search/components/HighlightedText";
import { cn } from "@/lib/utils";
import type { ClipboardItem, ClipboardItemType } from "@/types/clipboard";

const TYPE_ICONS: Record<ClipboardItemType, typeof FileText> = {
  text: FileText,
  image: Image,
  file: Files,
  url: Link2,
  code: Code2,
};

interface ClipboardCardProps {
  item: ClipboardItem;
  query: string;
  selected: boolean;
  onSelect: () => void;
  onCopy: () => void;
  onPin: () => void;
  onDelete: () => void;
}

export function ClipboardCard({
  item,
  query,
  selected,
  onSelect,
  onCopy,
  onPin,
  onDelete,
}: ClipboardCardProps) {
  const Icon = TYPE_ICONS[item.type];
  const time = new Date(item.createdAt).toLocaleString("fa-IR");
  const imageSrc = item.type === "image" ? clipboardImageSrc(item.content) : null;
  const [imageFailed, setImageFailed] = useState(false);

  return (
    <article
      role="option"
      aria-selected={selected}
      onClick={onSelect}
      onDoubleClick={onCopy}
      className={cn(
        "cursor-pointer rounded-[16px] border bg-[#181C23]/80 p-3 backdrop-blur-xl transition-[border-color,background-color,box-shadow] duration-[180ms]",
        selected
          ? "border-[#22C55E]/70 shadow-[0_0_0_1px_rgb(34_197_94_/_0.25)]"
          : "border-white/10 hover:border-white/20",
      )}
    >
      <div className="flex items-start gap-3">
        <Icon className="mt-0.5 size-4 shrink-0 text-[#22C55E]" />
        <div className="min-w-0 flex-1">
          {imageSrc && !imageFailed ? (
            <img
              src={imageSrc}
              alt=""
              loading="lazy"
              decoding="async"
              onError={() => setImageFailed(true)}
              className="mb-2 max-h-24 w-full rounded-[12px] object-contain bg-black/20"
            />
          ) : null}
          <p className="line-clamp-3 text-sm leading-6 text-[#F3F4F6]">
            <HighlightedText text={item.preview} query={query} />
          </p>
          <p className="mt-2 text-xs text-[#9CA3AF]">{time}</p>
        </div>
        <div className="flex shrink-0 items-center gap-1">
          <IconButton
            label={item.pinned ? "برداشتن سنجاق" : "سنجاق"}
            onClick={onPin}
          >
            <Pin className={cn("size-4", item.pinned && "fill-[#22C55E] text-[#22C55E]")} />
          </IconButton>
          <IconButton label="کپی" onClick={onCopy}>
            <Copy className="size-4" />
          </IconButton>
          <IconButton label="حذف" onClick={onDelete}>
            <Trash2 className="size-4" />
          </IconButton>
        </div>
      </div>
    </article>
  );
}

function IconButton({
  label,
  onClick,
  children,
}: {
  label: string;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          aria-label={label}
          onClick={(event) => {
            event.stopPropagation();
            onClick();
          }}
          className="text-[#9CA3AF] hover:bg-white/5 hover:text-[#F3F4F6]"
        >
          {children}
        </Button>
      </TooltipTrigger>
      <TooltipContent side="bottom">{label}</TooltipContent>
    </Tooltip>
  );
}
