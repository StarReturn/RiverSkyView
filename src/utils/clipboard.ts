import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";

/**
 * 剪贴板管理器：支持条件性自动清理
 * 仅当剪贴板内容仍为本应用写入的内容时才清空
 */
export class ClipboardManager {
  private static timer: ReturnType<typeof setTimeout> | null = null;
  private static writtenContent: string | null = null;

  static async write(text: string, autoClearSeconds: number = 30): Promise<void> {
    this.writtenContent = text;
    await writeText(text);

    this.clearTimer();

    if (autoClearSeconds > 0) {
      this.timer = setTimeout(async () => {
        await this.maybeClear();
      }, autoClearSeconds * 1000);
    }
  }

  /**
   * 仅当剪贴板内容仍是本应用写入的内容时才清空
   */
  static async maybeClear(): Promise<void> {
    if (this.writtenContent === null) return;

    try {
      const current = await readText();
      if (current === this.writtenContent) {
        await writeText("");
      }
    } catch {
      // 读取失败，不清空
    }

    this.writtenContent = null;
    this.clearTimer();
  }

  static cancelAutoClear(): void {
    this.clearTimer();
    this.writtenContent = null;
  }

  private static clearTimer(): void {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }
}
