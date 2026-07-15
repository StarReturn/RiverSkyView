import { invokeCommand } from "./index";
import type { FileNode, FilePreview } from "@/types";

export const filesApi = {
  listDirectory: (projectId: string, relativeDir?: string) =>
    invokeCommand<FileNode[]>("list_directory", { projectId, relativeDir }),

  readFileForPreview: (projectId: string, relativePath: string) =>
    invokeCommand<FilePreview>("read_file_for_preview", { projectId, relativePath }),

  getAbsolutePath: (projectId: string, relativePath: string) =>
    invokeCommand<string>("get_absolute_path", { projectId, relativePath }),

  openInExplorer: (projectId: string, relativePath?: string) =>
    invokeCommand<void>("open_in_explorer", { projectId, relativePath }),

  revealInExplorer: (projectId: string, relativePath: string) =>
    invokeCommand<void>("reveal_in_explorer", { projectId, relativePath }),

  copyFileToClipboard: (projectId: string, relativePath: string) =>
    invokeCommand<void>("copy_file_to_clipboard", { projectId, relativePath }),
};
