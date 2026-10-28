export type GameStatus = "idle" | "launching" | "running"

export type UpscaleMethod = "fsr" | "nis" | "integer" | "pixel" | "stretch"
export type WindowType = "borderless" | "fullscreen"

export interface EnvVar {
  id: string
  key: string
  value: string
}

export interface GamescopeConfig {
  renderW: string
  renderH: string
  outW: string
  outH: string
  method: UpscaleMethod
  windowType: WindowType
  grabCursor: boolean
  fps: string
  fpsUnfocused: string
  extra: string
}

export interface GraphicsConfig {
  gamemode: boolean
  mangohud: boolean
  gamescope: boolean
  dlss: boolean
  fsr3: boolean
  fsr4: boolean
  fsr4rdna3: boolean
  xess: boolean
  gs: GamescopeConfig
}

export interface WineConfig {
  nvapi: boolean
  wayland: boolean
  hdr: boolean
  wow64: boolean
  wined3d: boolean
  log: boolean
  noSteamClient: boolean
  skipRuntimeUpdate: boolean
  bwrapLock: boolean
}

export interface LaunchConfig {
  args: string
  dllOverrides: string
  wrapper: string
  env: EnvVar[]
}

export interface Game {
  id: string
  title: string
  exe: string
  installDir: string
  prefix: string
  runner: string
  cover?: string
  hero?: string
  hue: number
  playtimeMin: number
  lastPlayedDaysAgo: number | null
  status: GameStatus
  shortcut: boolean
  prefixSize: string | null
  launch: LaunchConfig
  graphics: GraphicsConfig
  wine: WineConfig
}

export type RunnerSource = "ge" | "umu" | "cachyos"

export interface Runner {
  id: string
  name: string
  source: RunnerSource
  version: string
  size: string
  date: string
  installed?: boolean
  sizeBytes?: number
}

export interface Download {
  runnerId: string
  progress: number
}

export type View = "library" | "cabinet" | "settings"

export type ThemeId = "crucible" | "midnight" | "catppuccin" | "nord"
export type FontId = "geist" | "inter" | "mono"
export type LogLevel = "error" | "warn" | "info" | "debug" | "trace"

export interface AppSettings {
  minimizeToTray: boolean
  rememberGeometry: boolean
  autoUpdateUmu: boolean
  font: FontId
  theme: ThemeId
  protonDir: string
  defaultRunner: string
  defaultArgs: string
  defaultDll: string
  defaultWrapper: string
  defaultGamemode: boolean
  defaultMangohud: boolean
  defaultGamescope: boolean
  logLevel: LogLevel
}

export type ToastTone = "success" | "info" | "danger" | "launch"

export interface Toast {
  id: string
  title: string
  description?: string
  tone: ToastTone
}

export interface BackupInfo {
  file_path: string
  file_name: string
  size_bytes: number
  size_formatted: string
  timestamp: string
}

export interface ZipApplyResult {
  success: boolean
  dlls: string[]
  exe: string
  message: string
}
