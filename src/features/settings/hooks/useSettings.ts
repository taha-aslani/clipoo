import { useEffect, useState } from "react";
import { isTauriRuntime } from "@/lib/runtime";
import { getSettings, updateSettings } from "@/lib/api/settings";
import { DEFAULT_SETTINGS, type AppSettings } from "@/types/settings";

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }

    let cancelled = false;
    void getSettings()
      .then((value) => {
        if (!cancelled) {
          setSettings(value);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setSettings(DEFAULT_SETTINGS);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const save = async (next: AppSettings) => {
    if (!isTauriRuntime()) {
      setSettings(next);
      return next;
    }

    const saved = await updateSettings(next);
    setSettings(saved);
    return saved;
  };

  return { settings, save };
}
