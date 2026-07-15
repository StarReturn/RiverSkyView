import { invokeCommand } from "./index";
import type {
  Project,
  ProjectListItem,
  AddProjectRequest,
  AddProjectResult,
  RenameProjectRequest,
} from "@/types";

export const projectApi = {
  add: (request: AddProjectRequest) =>
    invokeCommand<AddProjectResult>("add_project", { request }),

  list: () =>
    invokeCommand<ProjectListItem[]>("list_projects"),

  listFavorites: () =>
    invokeCommand<ProjectListItem[]>("list_favorite_projects"),

  listRecent: (limit?: number) =>
    invokeCommand<ProjectListItem[]>("list_recent_projects", { limit }),

  listRemoved: () =>
    invokeCommand<ProjectListItem[]>("list_removed_projects"),

  search: (query: string) =>
    invokeCommand<ProjectListItem[]>("search_projects", { query }),

  get: (projectId: string) =>
    invokeCommand<Project>("get_project", { projectId }),

  rename: (request: RenameProjectRequest) =>
    invokeCommand<Project>("rename_project", { request }),

  setFavorite: (projectId: string, favorite: boolean) =>
    invokeCommand<void>("set_project_favorite", { projectId, favorite }),

  remove: (projectId: string) =>
    invokeCommand<void>("remove_project", { projectId }),

  batchRemove: (projectIds: string[]) =>
    invokeCommand<number>("batch_remove_projects", { projectIds }),

  restore: (projectId: string) =>
    invokeCommand<Project>("restore_project", { projectId }),

  checkInstructionFiles: (projectId: string) =>
    invokeCommand<Record<string, unknown>>("check_instruction_files", { projectId }),

  count: () =>
    invokeCommand<number>("count_projects"),
};
