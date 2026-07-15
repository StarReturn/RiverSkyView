import { invokeCommand } from "./index";
import type { LaunchRequest, LaunchResult, ToolAvailability } from "@/types";

export const terminalApi = {
  launch: (request: LaunchRequest) =>
    invokeCommand<LaunchResult>("launch_terminal", { request }),

  checkTools: () =>
    invokeCommand<ToolAvailability[]>("check_tools_availability"),

  launchCodex: (projectId: string, action?: string) =>
    invokeCommand<LaunchResult>("launch_codex", { projectId, action }),

  launchClaude: (projectId: string, action?: string) =>
    invokeCommand<LaunchResult>("launch_claude", { projectId, action }),

  setProjectDefaultAction: (projectId: string, toolKind: string, action: string) =>
    invokeCommand<void>("set_project_default_action", { projectId, toolKind, action }),

  getProjectDefaultAction: (projectId: string) =>
    invokeCommand<{ codex: string; claude: string }>("get_project_default_action", { projectId }),
};
