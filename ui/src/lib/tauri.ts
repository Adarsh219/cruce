import type { AppSettings, BackupInfo, Game, Runner, ZipApplyResult } from "./types"

export function isTauri(): boolean {
  return typeof window !== "undefined" && Boolean((window as any).__TAURI_INTERNALS__)
}

export async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  if (!isTauri()) return null
  try {
    const { invoke } = await import("@tauri-apps/api/core")
    return await invoke<T>(cmd, args)
  } catch (err) {
    console.error(`[Tauri IPC] ${cmd} failed:`, err)
    throw err
  }
}

export async function tauriListen<T>(event: string, handler: (payload: T) => void): Promise<(() => void) | null> {
  if (!isTauri()) return null
  try {
    const { listen } = await import("@tauri-apps/api/event")
    const unlisten = await listen<T>(event, (e) => handler(e.payload))
    return unlisten
  } catch (err) {
    console.error(`[Tauri Event] ${event} listen failed:`, err)
    return null
  }
}

export async function minimizeWindow(): Promise<void> {
  if (isTauri()) {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window")
      await getCurrentWindow().minimize()
    } catch (e) {
      console.warn("Failed to minimize window:", e)
    }
  }
}

export async function toggleMaximizeWindow(): Promise<void> {
  if (isTauri()) {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window")
      await getCurrentWindow().toggleMaximize()
    } catch (e) {
      console.warn("Failed to toggle maximize window:", e)
    }
  }
}

export async function closeWindow(): Promise<void> {
  if (isTauri()) {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window")
      await getCurrentWindow().close()
    } catch (e) {
      console.warn("Failed to close window:", e)
    }
  }
}

export async function browseFile(filterName?: string, extensions?: string[]): Promise<string | null> {
  return await tauriInvoke<string | null>("browse_file", { filterName, extensions })
}

export async function browseDirectory(): Promise<string | null> {
  return await tauriInvoke<string | null>("browse_directory")
}

export async function openFolder(path: string): Promise<void> {
  await tauriInvoke("open_folder", { path })
}

export async function openWineTool(id: string, tool: string): Promise<void> {
  await tauriInvoke("open_wine_tool", { id, tool })
}

export async function resetPrefix(id: string): Promise<void> {
  await tauriInvoke("reset_prefix", { id })
}

export async function toggleShortcut(id: string): Promise<boolean | null> {
  return await tauriInvoke<boolean>("toggle_shortcut", { id })
}

export async function getGameLogs(id: string): Promise<[string, string][] | null> {
  return await tauriInvoke<[string, string][]>("get_game_logs", { id })
}

export async function backupGameSaves(id: string): Promise<BackupInfo | null> {
  return await tauriInvoke<BackupInfo>("backup_game_saves", { id })
}

export async function restoreGameSaves(id: string): Promise<void> {
  await tauriInvoke("restore_game_saves", { id })
}

export async function getBackupInfo(id: string): Promise<BackupInfo | null> {
  return await tauriInvoke<BackupInfo | null>("get_backup_info", { id })
}

export async function rescanLibrary(): Promise<Game[] | null> {
  return await tauriInvoke<Game[]>("rescan_library")
}

export async function getRunners(): Promise<Runner[] | null> {
  return await tauriInvoke<Runner[]>("get_runners")
}

export async function clearGameLogs(id: string): Promise<void> {
  await tauriInvoke("clear_game_logs", { id })
}

export async function openGameLogDir(id: string): Promise<void> {
  await tauriInvoke("open_game_log_dir", { id })
}

export async function renameGame(id: string, newTitle: string): Promise<Game | null> {
  return await tauriInvoke<Game>("rename_game", { id, newTitle })
}

export async function applyZipToGame(id: string, zipPath: string): Promise<ZipApplyResult | null> {
  return await tauriInvoke<ZipApplyResult>("apply_zip_to_game", { id, zipPath })
}
