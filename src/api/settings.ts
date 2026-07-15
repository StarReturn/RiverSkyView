import { invokeCommand } from "./index";
import type { AppSettings, SettingUpdate } from "@/types";

export const settingsApi = {
  get: () => invokeCommand<AppSettings>("get_settings"),

  update: (key: string, value: string) =>
    invokeCommand<AppSettings>("update_setting", { key, value }),

  updateMany: (updates: SettingUpdate[]) =>
    invokeCommand<AppSettings>("update_settings", { updates }),

  applySettingSideEffects: (key: string) =>
    invokeCommand<void>("apply_setting_side_effects", { key }),

  getDataDir: () => invokeCommand<string>("get_data_dir"),

  openDataDir: () => invokeCommand<void>("open_data_dir"),
};
