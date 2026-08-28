export interface AppSettings {
  launchOnStartup: boolean;
  enableMonitoring: boolean;
  maxHistorySize: number;
}

export const DEFAULT_SETTINGS: AppSettings = {
  launchOnStartup: true,
  enableMonitoring: true,
  maxHistorySize: 10_000,
};
