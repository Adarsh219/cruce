import type { AppSettings, Game, GraphicsConfig, Runner, RunnerSource, WineConfig } from "./types"

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

interface Seed {
  id: string
  title: string
  folder: string
  exe: string
  art?: string
  hue: number
  playtimeMin: number
  lastPlayedDaysAgo: number | null
  runner: string
  prefixSize: string | null
  shortcut?: boolean
  graphics?: Partial<GraphicsConfig>
  wine?: Partial<WineConfig>
  args?: string
  dll?: string
}

const seeds: Seed[] = [
  {
    id: "cp2077",
    title: "Cyberpunk 2077",
    folder: "Cyberpunk 2077",
    exe: "bin/x64/Cyberpunk2077.exe",
    art: "neon-city",
    hue: 60,
    playtimeMin: 860,
    lastPlayedDaysAgo: 1,
    runner: "GE-Proton10-21",
    prefixSize: "1.42 GB",
    shortcut: true,
    graphics: { mangohud: true, dlss: true },
    wine: { nvapi: true, hdr: true },
    args: "--launcher-skip -skipStartScreen",
  },
  {
    id: "elden",
    title: "Elden Ring",
    folder: "ELDEN RING",
    exe: "Game/eldenring.exe",
    art: "elden",
    hue: 45,
    playtimeMin: 7412,
    lastPlayedDaysAgo: 0,
    runner: "GE-Proton10-21",
    prefixSize: "812 MB",
    graphics: { gamescope: true },
    wine: { bwrapLock: true },
  },
  {
    id: "bg3",
    title: "Baldur's Gate 3",
    folder: "Baldurs Gate 3",
    exe: "bin/bg3_dx11.exe",
    art: "baldurs",
    hue: 300,
    playtimeMin: 5235,
    lastPlayedDaysAgo: 3,
    runner: "UMU-Proton-9.0-4e",
    prefixSize: "2.06 GB",
    shortcut: true,
    dll: "dxgi=n,b",
  },
  {
    id: "hades2",
    title: "Hades II",
    folder: "Hades II",
    exe: "Ship/Hades2.exe",
    art: "hades",
    hue: 150,
    playtimeMin: 1984,
    lastPlayedDaysAgo: 0,
    runner: "proton-cachyos-10.0-20251020",
    prefixSize: "468 MB",
  },
  {
    id: "starfield",
    title: "Starfield",
    folder: "Starfield",
    exe: "Starfield.exe",
    art: "starfield",
    hue: 210,
    playtimeMin: 342,
    lastPlayedDaysAgo: 14,
    runner: "GE-Proton10-20",
    prefixSize: null,
    graphics: { fsr3: true },
  },
  {
    id: "disco",
    title: "Disco Elysium",
    folder: "Disco Elysium",
    exe: "disco.exe",
    hue: 18,
    playtimeMin: 0,
    lastPlayedDaysAgo: null,
    runner: "GE-Proton10-21",
    prefixSize: null,
  },
]

export const createGame = (s: Seed): Game => ({
  id: s.id,
  title: s.title,
  exe: `~/Games/${s.folder}/${s.exe}`,
  installDir: `~/Games/${s.folder}`,
  prefix: `~/.local/share/crucible-launcher/Prefix/${s.id}`,
  runner: s.runner,
  cover: s.art ? `/games/${s.art}-cover.png` : undefined,
  hero: s.art ? `/games/${s.art}-hero.png` : undefined,
  hue: s.hue,
  playtimeMin: s.playtimeMin,
  lastPlayedDaysAgo: s.lastPlayedDaysAgo,
  status: "idle",
  shortcut: s.shortcut ?? false,
  prefixSize: s.prefixSize,
  launch: {
    args: s.args ?? "",
    dllOverrides: s.dll ?? "",
    wrapper: "",
    env: s.id === "cp2077" ? [{ id: "env-1", key: "DXVK_ASYNC", value: "1" }] : [],
  },
  graphics: { ...defaultGraphics(), ...s.graphics },
  wine: { ...defaultWine(), ...s.wine },
})

export const seedGames = (): Game[] => seeds.map(createGame)

export const releases: Runner[] = [
  { id: "ge-10-21", name: "GE-Proton10-21", source: "ge", version: "10-21", size: "452 MB", date: "Oct 22, 2026" },
  { id: "ge-10-20", name: "GE-Proton10-20", source: "ge", version: "10-20", size: "449 MB", date: "Oct 08, 2026" },
  { id: "ge-10-17", name: "GE-Proton10-17", source: "ge", version: "10-17", size: "447 MB", date: "Sep 19, 2026" },
  { id: "ge-10-16", name: "GE-Proton10-16", source: "ge", version: "10-16", size: "445 MB", date: "Sep 02, 2026" },
  { id: "ge-9-27", name: "GE-Proton9-27", source: "ge", version: "9-27", size: "431 MB", date: "Jun 14, 2026" },
  { id: "umu-9-4e", name: "UMU-Proton-9.0-4e", source: "umu", version: "9.0-4e", size: "418 MB", date: "Oct 11, 2026" },
  { id: "umu-9-4d", name: "UMU-Proton-9.0-4d", source: "umu", version: "9.0-4d", size: "417 MB", date: "Aug 27, 2026" },
  { id: "umu-9-3", name: "UMU-Proton-9.0-3", source: "umu", version: "9.0-3", size: "412 MB", date: "Jul 05, 2026" },
  { id: "cachy-1020", name: "proton-cachyos-10.0-20251020", source: "cachyos", version: "10.0-20251020", size: "476 MB", date: "Oct 20, 2026" },
  { id: "cachy-0928", name: "proton-cachyos-10.0-20250928", source: "cachyos", version: "10.0-20250928", size: "474 MB", date: "Sep 28, 2026" },
  { id: "cachy-0901", name: "proton-cachyos-10.0-20250901", source: "cachyos", version: "10.0-20250901", size: "470 MB", date: "Sep 01, 2026" },
]

export const initialInstalled = ["ge-10-21", "ge-10-20", "umu-9-4e", "cachy-1020"]

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

export const mockExecutables = [
  { title: "Hollow Knight", folder: "Hollow Knight", exe: "hollow_knight.exe", hue: 220 },
  { title: "Sekiro: Shadows Die Twice", folder: "Sekiro", exe: "sekiro.exe", hue: 0 },
  { title: "The Witcher 3", folder: "The Witcher 3", exe: "bin/x64_dx12/witcher3.exe", hue: 100 },
  { title: "Outer Wilds", folder: "Outer Wilds", exe: "OuterWilds.exe", hue: 30 },
  { title: "Celeste", folder: "Celeste", exe: "Celeste.exe", hue: 330 },
]
