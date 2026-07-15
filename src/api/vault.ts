import { invokeCommand } from "./index";
import type {
  VaultEntry,
  VaultListItem,
  VaultCreateRequest,
  VaultUpdateRequest,
  VaultImportRequest,
  VaultContent,
} from "@/types";

export const vaultApi = {
  create: (request: VaultCreateRequest) =>
    invokeCommand<VaultEntry>("create_vault_entry", { request }),

  importTxt: (request: VaultImportRequest) =>
    invokeCommand<VaultEntry>("import_vault_txt", { request }),

  getContent: (id: string) =>
    invokeCommand<VaultContent>("get_vault_content", { id }),

  update: (request: VaultUpdateRequest) =>
    invokeCommand<VaultEntry>("update_vault_entry", { request }),

  list: () =>
    invokeCommand<VaultListItem[]>("list_vault_entries"),

  listRemoved: () =>
    invokeCommand<VaultListItem[]>("list_removed_vault_entries"),

  search: (query: string) =>
    invokeCommand<VaultListItem[]>("search_vault_entries", { query }),

  remove: (id: string) =>
    invokeCommand<void>("remove_vault_entry", { id }),

  restore: (id: string) =>
    invokeCommand<void>("restore_vault_entry", { id }),

  permanentDelete: (id: string) =>
    invokeCommand<void>("permanent_delete_vault_entry", { id }),

  clearPlaintext: () =>
    invokeCommand<void>("clear_vault_plaintext"),
};
