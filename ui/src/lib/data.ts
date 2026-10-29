import type { AppSettings, GraphicsConfig, RunnerSource, WineConfig } from "./types"

export const defaultGraphics = (): GraphicsConfig => ({
  gamemode: true,
  mangohud: false,
  gamescope: false,
  dlss: false,
  fsr3: false,
  fsr4: false,
  fsr4rdna3: false,
  xess: false,
  gs: {
    renderW: "1920",
    renderH: "1080",
    outW: "2560",
    outH: "1440",
    method: "fsr",
    windowType: "borderless",
    grabCursor: false,
    fps: "",
    fpsUnfocused: "30",
    extra: "",
  },
})

export const defaultWine = (): WineConfig => ({
  nvapi: false,
  wayland: false,
  hdr: false,
  wow64: false,
  wined3d: false,
  log: false,
  noSteamClient: false,
  skipRuntimeUpdate: false,
  bwrapLock: false,
})

export const sourceMeta: Record<RunnerSource, { label: string; repo: string }> = {
  ge: { label: "GE-Proton", repo: "GloriousEggroll/proton-ge-custom" },
  umu: { label: "UMU-Proton", repo: "Open-Wine-Components/umu-proton" },
  cachyos: { label: "CachyOS Proton", repo: "CachyOS/proton-cachyos" },
}

export const defaultSettings: AppSettings = {
  minimizeToTray: true,
  rememberGeometry: true,
  autoUpdateUmu: true,
  font: "geist",
  theme: "crucible",
  protonDir: "~/.local/share/Steam/compatibilitytools.d",
  defaultRunner: "GE-Proton10-21",
  defaultArgs: "",
  defaultDll: "",
  defaultWrapper: "",
  defaultGamemode: true,
  defaultMangohud: false,
  defaultGamescope: false,
  logLevel: "info",
}
