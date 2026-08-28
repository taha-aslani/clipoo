import { useEffect, useState, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { useSettings } from "@/features/settings/hooks/useSettings";
import { clearClipboardHistory } from "@/lib/api/clipboard";
import { isTauriRuntime } from "@/lib/runtime";

interface SettingsPanelProps {
  onBack: () => void;
  onHistoryCleared: () => void;
}

export function SettingsPanel({ onBack, onHistoryCleared }: SettingsPanelProps) {
  const { settings, save } = useSettings();
  const [historyInput, setHistoryInput] = useState(String(settings.maxHistorySize));
  const [confirmClear, setConfirmClear] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    setHistoryInput(String(settings.maxHistorySize));
  }, [settings.maxHistorySize]);

  const commitHistorySize = async () => {
    const parsed = Number.parseInt(historyInput, 10);
    const maxHistorySize = Number.isFinite(parsed) ? Math.max(1, parsed) : settings.maxHistorySize;
    setHistoryInput(String(maxHistorySize));
    if (maxHistorySize !== settings.maxHistorySize) {
      try {
        await save({ ...settings, maxHistorySize });
      } catch {
        setHistoryInput(String(settings.maxHistorySize));
      }
    }
  };

  const clearHistory = async () => {
    if (!confirmClear) {
      setConfirmClear(true);
      return;
    }

    setBusy(true);
    try {
      if (isTauriRuntime()) {
        await clearClipboardHistory();
      }
      setConfirmClear(false);
      onHistoryCleared();
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="flex min-h-0 flex-1 flex-col">
      <div className="sticky top-0 z-10 flex items-center justify-between border-b border-white/5 bg-[#0F1115]/80 px-4 py-3 backdrop-blur-xl">
        <h1 className="text-sm font-medium">تنظیمات</h1>
        <Button
          type="button"
          variant="ghost"
          onClick={onBack}
          className="h-9 rounded-[16px] text-[#9CA3AF] hover:text-[#F3F4F6]"
        >
          بازگشت
        </Button>
      </div>

      <div className="flex-1 space-y-6 overflow-y-auto p-4">
        <div className="rounded-[16px] border border-white/10 bg-[#181C23]/80 p-3">
          <SettingRow
            title="اجرا هنگام روشن شدن ویندوز"
            description="Clipoo بعد از ورود به ویندوز باز می‌شود."
          >
            <Switch
              checked={settings.launchOnStartup}
              onCheckedChange={(launchOnStartup) => {
                void save({ ...settings, launchOnStartup }).catch(() => undefined);
              }}
            />
          </SettingRow>
          <Separator className="my-3 bg-white/5" />
          <SettingRow
            title="پایش کلیپ‌بورد"
            description="موارد جدید به‌صورت خودکار ذخیره می‌شوند."
          >
            <Switch
              checked={settings.enableMonitoring}
              onCheckedChange={(enableMonitoring) => {
                void save({ ...settings, enableMonitoring }).catch(() => undefined);
              }}
            />
          </SettingRow>
        </div>

        <div className="rounded-[16px] border border-white/10 bg-[#181C23]/80 p-3">
          <label htmlFor="max-history" className="block text-sm font-medium">
            حداکثر تعداد تاریخچه
          </label>
          <p className="mt-1 text-xs text-[#9CA3AF]">
            قدیمی‌ترین موارد سنجاق‌نشده پس از رسیدن به این عدد حذف می‌شوند.
          </p>
          <Input
            id="max-history"
            dir="ltr"
            inputMode="numeric"
            value={historyInput}
            onChange={(event) => setHistoryInput(event.currentTarget.value)}
            onBlur={() => {
              void commitHistorySize();
            }}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.currentTarget.blur();
              }
            }}
            className="mt-3 h-11 rounded-[16px] border-white/10 bg-[#0F1115] text-sm"
          />
        </div>

        <div className="rounded-[16px] border border-white/10 bg-[#181C23]/80 p-3">
          <p className="text-sm font-medium">پاک کردن تاریخچه</p>
          <p className="mt-1 text-xs text-[#9CA3AF]">
            همه موارد برای همیشه حذف می‌شوند. این کار قابل بازگشت نیست.
          </p>
          <Button
            type="button"
            variant="destructive"
            disabled={busy}
            onClick={() => {
              void clearHistory();
            }}
            className="mt-3 h-10 rounded-[16px]"
          >
            {confirmClear ? "تأیید پاک کردن" : "پاک کردن تاریخچه"}
          </Button>
        </div>
      </div>
    </section>
  );
}

function SettingRow({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="min-w-0">
        <p className="text-sm">{title}</p>
        <p className="mt-1 text-xs text-[#9CA3AF]">{description}</p>
      </div>
      {children}
    </div>
  );
}
