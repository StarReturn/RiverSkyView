import { invokeCommand } from "./index";
import type { ProjectLog, LogSyncResult, ActivitySummary } from "@/types";

export const projectLogApi = {
  sync: (projectId: string) =>
    invokeCommand<LogSyncResult>("sync_project_logs", { projectId }),

  list: (
    projectId: string,
    agent?: string,
    status?: string,
    date?: string,
  ) =>
    invokeCommand<ProjectLog[]>("list_project_logs", { projectId, agent, status, date }),

  getContent: (projectId: string, logId: string) =>
    invokeCommand<string | null>("get_log_content", { projectId, logId }),

  getActivitySummary: (projectId: string, days?: number) =>
    invokeCommand<ActivitySummary>("get_activity_summary", { projectId, days }),
};
