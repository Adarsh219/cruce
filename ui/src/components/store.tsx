import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react"
import { defaultGraphics, defaultSettings, defaultWine } from "@/lib/data"
import { useGamepadNavigation, type GamepadState } from "@/lib/gamepad"
import { getRunners, isTauri, rescanLibrary, tauriInvoke, tauriListen } from "@/lib/tauri"
import type { AppSettings, Download, Game, Runner, Toast, ToastTone, View } from "@/lib/types"

interface CruceState {
  view: View
  setView: (v: View) => void
  games: Game[]
  updateGame: (id: string, fn: (g: Game) => Game) => void
  addGame: (input: { title: string; exe: string; installDir: string; hue: number; runner: string }) => void
  removeGame: (id: string) => void
  rescan: () => Promise<void>
  openGameId: string | null
  setOpenGameId: (id: string | null) => void
  togglePlay: (id: string) => void
  addDialogOpen: boolean
  setAddDialogOpen: (v: boolean) => void
  runners: Runner[]
  refreshRunners: () => Promise<void>
  installed: string[]
  removeRunner: (id: string) => void
  download: Download | null
  startDownload: (runnerId: string) => void
  cancelDownload: () => void
  settings: AppSettings
  updateSettings: (patch: Partial<AppSettings>) => void
  toasts: Toast[]
  toast: (title: string, opts?: { description?: string; tone?: ToastTone }) => void
  dismissToast: (id: string) => void
  installedRunnerNames: string[]
  gamepad: GamepadState & { setFocusedIndex: (i: number) => void }
}

const Ctx = createContext<CruceState | null>(null)

export function useCruce() {
  const ctx = useContext(Ctx)
  if (!ctx) throw new Error("useCruce must be used within CruceProvider")
  return ctx
}

export function CruceProvider({ children }: { children: React.ReactNode }) {
  const [view, setView] = useState<View>("library")
  const [games, setGames] = useState<Game[]>([])
  const [runners, setRunners] = useState<Runner[]>([])
  const [openGameId, setOpenGameId] = useState<string | null>(null)
  const [addDialogOpen, setAddDialogOpen] = useState(false)
  const [installed, setInstalled] = useState<string[]>([])
  const [download, setDownload] = useState<Download | null>(null)
  const [settings, setSettings] = useState<AppSettings>(defaultSettings)
  const [toasts, setToasts] = useState<Toast[]>([])
  const idRef = useRef(0)
  const nextId = useCallback((p: string) => `${p}-${++idRef.current}`, [])

  const dismissToast = useCallback((id: string) => {
    setToasts((t) => t.filter((x) => x.id !== id))
  }, [])

  const toast = useCallback<CruceState["toast"]>(
    (title, opts) => {
      const id = nextId("toast")
      setToasts((t) => [...t.slice(-3), { id, title, description: opts?.description, tone: opts?.tone ?? "success" }])
      setTimeout(() => dismissToast(id), 3600)
    },
    [nextId, dismissToast],
  )

  const gamesRef = useRef(games)
  gamesRef.current = games

  const refreshRunners = useCallback(async () => {
    if (!isTauri()) return
    try {
      const list = await getRunners()
      if (list) {
        setRunners(list)
        setInstalled(list.filter((r) => r.installed).map((r) => r.id))
      }
    } catch (e) {
      console.error("Error loading runners:", e)
    }
  }, [])

  // Load real data and wire real-time events when running in Tauri
  useEffect(() => {
    if (!isTauri()) return

    tauriInvoke<Game[]>("get_games")
      .then((loaded) => {
        setGames(loaded || [])
      })
      .catch((e) => console.error("Error loading games:", e))

    refreshRunners()

    tauriInvoke<AppSettings>("get_settings")
      .then((loadedSettings) => {
        if (loadedSettings) {
          setSettings(loadedSettings)
        }
      })
      .catch((e) => console.error("Error loading settings:", e))

    let unlistenState: (() => void) | null = null
    tauriListen<{ id: string; title: string; status: "idle" | "launching" | "running"; playtimeDeltaSeconds?: number }>(
      "game_state_changed",
      (payload) => {
        setGames((gs) =>
          gs.map((g) => {
            if (g.id === payload.id || g.title === payload.title) {
              const playtimeDeltaMin = payload.playtimeDeltaSeconds ? Math.round(payload.playtimeDeltaSeconds / 60) : 0
              return {
                ...g,
                status: payload.status,
                playtimeMin: g.playtimeMin + playtimeDeltaMin,
                ...(payload.status === "running" ? { lastPlayedDaysAgo: 0 } : {}),
              }
            }
            return g
          }),
        )
      },
    ).then((u) => {
      unlistenState = u
    })

    let unlistenDownload: (() => void) | null = null
    tauriListen<{ runnerId: string; progress: number; complete?: boolean; error?: string }>(
      "runner_download_progress",
      (payload) => {
        if (payload.complete) {
          setDownload(null)
          refreshRunners()
          toast("Download complete", { description: `${payload.runnerId} ready to use` })
        } else if (payload.error) {
          setDownload(null)
          toast("Download failed", { description: payload.error, tone: "danger" })
        } else {
          setDownload({ runnerId: payload.runnerId, progress: payload.progress })
        }
      },
    ).then((u) => {
      unlistenDownload = u
    })

    let unlistenArtwork: (() => void) | null = null
    tauriListen<{ id: string; title: string }>(
      "artwork_updated",
      (payload) => {
        tauriInvoke<Game>("get_game", { id: payload.id }).then((updated) => {
          if (updated) {
            setGames((gs) =>
              gs.map((g) =>
                g.id === updated.id || g.title === updated.title
                  ? { ...g, cover: updated.cover, hero: updated.hero }
                  : g,
              ),
            )
          }
        })
      },
    ).then((u) => {
      unlistenArtwork = u
    })

    return () => {
      unlistenState?.()
      unlistenDownload?.()
      unlistenArtwork?.()
    }
  }, [refreshRunners, toast])

  const updateGame = useCallback(
    (id: string, fn: (g: Game) => Game) => {
      setGames((gs) => {
        const next = gs.map((g) => (g.id === id ? fn(g) : g))
        const updated = next.find((g) => g.id === id)
        if (updated && isTauri()) {
          tauriInvoke("save_game", { game: updated }).catch((e) => {
            console.error("Failed to save game to disk:", e)
          })
        }
        return next
      })
    },
    [],
  )

  const togglePlay = useCallback(
    (id: string) => {
      const game = gamesRef.current.find((g) => g.id === id)
      if (!game) return

      if (!isTauri()) {
        console.warn("[Cruce] Game execution requires native Tauri runtime")
        return
      }

      if (game.status === "idle") {
        setGames((gs) => gs.map((g) => (g.id === id ? { ...g, status: "launching" } : g)))
        toast(`Launching ${game.title}…`, { description: `umu-run via ${game.runner}`, tone: "launch" })
        tauriInvoke("launch_game", { id: game.id }).catch((err) => {
          setGames((gs) => gs.map((g) => (g.id === id ? { ...g, status: "idle" } : g)))
          toast(`Launch failed for ${game.title}`, { description: String(err), tone: "danger" })
        })
      } else {
        setGames((gs) => gs.map((g) => (g.id === id ? { ...g, status: "idle" } : g)))
        toast(`${game.title} stopped`, { description: "Terminated Wine session", tone: "info" })
        tauriInvoke("stop_game", { id: game.id }).catch((err) => {
          toast(`Stop failed`, { description: String(err), tone: "danger" })
        })
      }
    },
    [toast],
  )

  useEffect(() => {
    const t = setInterval(() => {
      setGames((gs) =>
        gs.some((g) => g.status === "running")
          ? gs.map((g) => (g.status === "running" ? { ...g, playtimeMin: g.playtimeMin + 1 } : g))
          : gs,
      )
    }, 60_000)
    return () => clearInterval(t)
  }, [])

  const addGame = useCallback<CruceState["addGame"]>(
    (input) => {
      if (isTauri()) {
        tauriInvoke<Game>("add_game", input)
          .then((added) => {
            if (added) {
              setGames((gs) => [...gs, added])
              toast("Added game", { description: `${input.title} is ready to play` })
            }
          })
          .catch((err) => {
            toast("Failed to add game", { description: String(err), tone: "danger" })
          })
        return
      }

      const id = nextId("game")
      const g: Game = {
        id,
        title: input.title,
        exe: input.exe,
        installDir: input.installDir,
        prefix: `~/.local/share/cruce/prefixes/${id}`,
        runner: input.runner,
        hue: input.hue,
        playtimeMin: 0,
        lastPlayedDaysAgo: null,
        status: "idle",
        shortcut: false,
        prefixSize: null,
        launch: {
          args: settings.defaultArgs,
          dllOverrides: settings.defaultDll,
          wrapper: settings.defaultWrapper,
          env: [],
        },
        graphics: {
          ...defaultGraphics(),
          gamemode: settings.defaultGamemode,
          mangohud: settings.defaultMangohud,
          gamescope: settings.defaultGamescope,
        },
        wine: defaultWine(),
      }
      setGames((gs) => [...gs, g])
      toast("Added game", { description: `${input.title} is ready to play` })
    },
    [nextId, settings, toast],
  )

  const removeGame = useCallback(
    (id: string) => {
      const g = gamesRef.current.find((x) => x.id === id)
      if (isTauri()) {
        tauriInvoke("remove_game", { id }).catch((e) => console.error("remove_game failed:", e))
      }
      setGames((gs) => gs.filter((x) => x.id !== id))
      setOpenGameId(null)
      if (g) toast(`Removed ${g.title}`, { description: "Prefix and files left on disk", tone: "danger" })
    },
    [toast],
  )

  const rescan = useCallback(async () => {
    if (isTauri()) {
      try {
        const loaded = await rescanLibrary()
        if (loaded) {
          setGames(loaded)
          toast("Library rescanned", { description: `${loaded.length} games loaded` })
        }
      } catch (e) {
        toast("Rescan failed", { description: String(e), tone: "danger" })
      }
    } else {
      toast("Library rescanned", { description: `${games.length} games · artwork cache up to date` })
    }
  }, [games.length, toast])

  const removeRunner = useCallback(
    async (id: string) => {
      const r = runners.find((x) => x.id === id)
      if (isTauri()) {
        try {
          await tauriInvoke("remove_runner", { runnerId: id })
          await refreshRunners()
          if (r) toast(`Deleted ${r.name}`, { description: `Freed ${r.size}`, tone: "danger" })
        } catch (e) {
          toast("Failed to remove runner", { description: String(e), tone: "danger" })
        }
        return
      }
      setInstalled((i) => i.filter((x) => x !== id))
      if (r) toast(`Deleted ${r.name}`, { description: `Freed ${r.size}`, tone: "danger" })
    },
    [runners, refreshRunners, toast],
  )

  const cancelDownload = useCallback(() => {
    if (isTauri()) {
      tauriInvoke("cancel_download").catch((e) => console.error("cancel_download failed:", e))
    }
    setDownload(null)
    toast("Download cancelled", { tone: "info" })
  }, [toast])

  const startDownload = useCallback(
    (runnerId: string) => {
      const r = runners.find((x) => x.id === runnerId)
      if (!isTauri()) {
        toast("Native runtime required", { description: "Downloading runners requires running inside Cruce", tone: "info" })
        return
      }

      setDownload({ runnerId, progress: 0 })
      toast(`Downloading ${r?.name ?? runnerId}`, { description: "From GitHub Releases", tone: "info" })
      tauriInvoke("download_runner", { runnerId }).catch((err) => {
        setDownload(null)
        toast("Download failed", { description: String(err), tone: "danger" })
      })
    },
    [runners, toast],
  )

  const updateSettings = useCallback((patch: Partial<AppSettings>) => {
    setSettings((s) => {
      const updated = { ...s, ...patch }
      if (isTauri()) {
        tauriInvoke("update_settings", { settings: updated }).catch((e) => {
          console.error("update_settings failed:", e)
        })
      }
      return updated
    })
  }, [])

  const installedRunnerNames = useMemo(() => {
    return runners.filter((r) => r.installed || installed.includes(r.id)).map((r) => r.name)
  }, [runners, installed])

  const gamepad = useGamepadNavigation({
    games,
    view,
    setView,
    openGameId,
    setOpenGameId,
    addDialogOpen,
    setAddDialogOpen,
    togglePlay,
    toast,
  })

  const value: CruceState = {
    view,
    setView,
    games,
    updateGame,
    addGame,
    removeGame,
    rescan,
    openGameId,
    setOpenGameId,
    togglePlay,
    addDialogOpen,
    setAddDialogOpen,
    runners,
    refreshRunners,
    installed,
    removeRunner,
    download,
    startDownload,
    cancelDownload,
    settings,
    updateSettings,
    toasts,
    toast,
    dismissToast,
    installedRunnerNames,
    gamepad,
  }

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>
}
