import { Clock, FlaskConical, Gauge, HardDrive, Info, Loader2, MonitorPlay, Play, Rocket, ScrollText, Square, SquareArrowOutUpRight, Wrench, X } from "lucide-react"
import { useState } from "react"
import { formatLastPlayed, formatPlaytime } from "@/lib/format"
import type { Game } from "@/lib/types"
import { cn } from "@/lib/utils"
import { isTauri, openWineTool, toggleShortcut } from "@/lib/tauri"
import { Dialog } from "../dialog"
import { FallbackCover, GameIcon } from "../game-art"
import { useCruce } from "../store"
import { Pill } from "../ui"
import { AdvancedTab } from "./advanced-tab"
import { DataTab } from "./data-tab"
import { GraphicsTab } from "./graphics-tab"
import { LaunchTab } from "./launch-tab"
import { LogsDialog } from "./logs-dialog"
import { OverviewTab } from "./overview-tab"

type Tab = "overview" | "launch" | "graphics" | "advanced" | "data"

const tabs: { id: Tab; label: string; icon: typeof Info }[] = [
  { id: "overview", label: "Overview", icon: Info },
  { id: "launch", label: "Launch Options", icon: Rocket },
  { id: "graphics", label: "Graphics", icon: Gauge },
  { id: "advanced", label: "Advanced Wine", icon: FlaskConical },
  { id: "data", label: "Manage Data", icon: HardDrive },
]

export function GameModal() {
  const { openGameId, setOpenGameId, games } = useCruce()
  const game = games.find((g) => g.id === openGameId)
  return (
    <Dialog
      open={!!game}
      onClose={() => setOpenGameId(null)}
      label={game ? `${game.title} settings` : "Game settings"}
      className="h-full max-w-5xl sm:h-[min(92dvh,900px)]"
    >
      {game && <GameModalBody key={game.id} game={game} />}
    </Dialog>
  )
}

function GameModalBody({ game }: { game: Game }) {
  const { setOpenGameId, togglePlay, updateGame, toast } = useCruce()
  const [tab, setTab] = useState<Tab>("overview")
  const [logsOpen, setLogsOpen] = useState(false)
  const isRunning = game.status === "running"
  const isLaunching = game.status === "launching"

  return (
    <>
      <div className="min-h-0 flex-1 overflow-y-auto scrollbar-thin">
        <div className="relative h-56 shrink-0 overflow-hidden sm:h-72">
          {game.hero ? (
            <img src={game.hero} alt="" className="absolute inset-0 size-full object-cover" />
          ) : (
            <FallbackCover game={game} />
          )}
          <div className="absolute inset-0 bg-[linear-gradient(to_top,var(--background)_0%,color-mix(in_oklch,var(--background)_70%,transparent)_35%,transparent_75%)]" />
          <div className="absolute inset-0 bg-[linear-gradient(to_right,color-mix(in_oklch,var(--background)_60%,transparent),transparent_60%)]" />

          <button
            type="button"
            onClick={() => setOpenGameId(null)}
            aria-label="Close"
            className="absolute top-4 right-4 flex size-9 items-center justify-center rounded-full border border-white/15 bg-black/50 text-white backdrop-blur-md transition-colors hover:bg-black/80"
          >
            <X className="size-4" />
          </button>

          <div className="absolute inset-x-5 bottom-5 flex items-end gap-4 sm:inset-x-8">
            <GameIcon game={game} className="size-14 rounded-xl sm:size-16" />
            <div className="min-w-0">
              <div className="mb-1.5">
                {isRunning ? (
                  <Pill tone="success" pulse>
                    Running
                  </Pill>
                ) : isLaunching ? (
                  <Pill tone="warning" pulse>
                    Launching
                  </Pill>
                ) : (
                  <Pill>Ready</Pill>
                )}
              </div>
              <h2 className="truncate text-2xl font-bold tracking-tight text-foreground sm:text-4xl">{game.title}</h2>
            </div>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-3 border-b border-border px-5 pb-5 sm:gap-5 sm:px-8">
          <button
            type="button"
            onClick={() => togglePlay(game.id)}
            disabled={isLaunching}
            className={cn(
              "flex h-12 min-w-36 items-center justify-center gap-2.5 rounded-xl px-7 text-base font-bold tracking-wide transition-all outline-none focus-visible:ring-2 focus-visible:ring-ring active:scale-[0.97]",
              isRunning
                ? "bg-destructive text-destructive-foreground hover:brightness-110"
                : "bg-primary text-primary-foreground shadow-[0_10px_30px_-10px_var(--primary)] hover:brightness-110",
            )}
          >
            {isLaunching ? (
              <Loader2 className="size-5 animate-spin" />
            ) : isRunning ? (
              <Square className="size-4 fill-current" />
            ) : (
              <Play className="size-5 fill-current" />
            )}
            {isLaunching ? "LAUNCHING" : isRunning ? "STOP" : "PLAY"}
          </button>

          <dl className="flex gap-6">
            <div>
              <dt className="text-[11px] font-semibold tracking-widest text-muted-foreground uppercase">Last Played</dt>
              <dd className="mt-0.5 text-sm font-medium">{formatLastPlayed(game.lastPlayedDaysAgo)}</dd>
            </div>
            <div>
              <dt className="flex items-center gap-1 text-[11px] font-semibold tracking-widest text-muted-foreground uppercase">
                <Clock className="size-3" /> Play Time
              </dt>
              <dd className="mt-0.5 font-mono text-sm font-medium">{formatPlaytime(game.playtimeMin)}</dd>
            </div>
          </dl>

          <div className="flex w-full items-center gap-1 sm:ml-auto sm:w-auto">
            <ToolbarBtn
              icon={Wrench}
              label="Winetricks"
              onClick={() => {
                if (isTauri()) {
                  openWineTool(game.id, "winetricks")
                    .then(() => toast("Opening Winetricks", { description: `WINEPREFIX=${game.prefix.split("/").pop()}`, tone: "info" }))
                    .catch((e) => toast("Failed to open Winetricks", { description: String(e), tone: "danger" }))
                  return
                }
                toast("Opening Winetricks", { description: `WINEPREFIX=${game.prefix.split("/").pop()}`, tone: "info" })
              }}
            />
            <ToolbarBtn icon={ScrollText} label="Logs" onClick={() => setLogsOpen(true)} />
            <ToolbarBtn
              icon={game.shortcut ? SquareArrowOutUpRight : MonitorPlay}
              label={game.shortcut ? "Remove shortcut" : "Add shortcut"}
              active={game.shortcut}
              onClick={() => {
                if (isTauri()) {
                  toggleShortcut(game.id).then((active) => {
                    if (active !== null) {
                      updateGame(game.id, (g) => ({ ...g, shortcut: active }))
                      toast(active ? "Desktop shortcut created" : "Desktop shortcut removed", {
                        description: `crucible-${game.id}.desktop`,
                        tone: active ? "success" : "danger",
                      })
                    }
                  })
                } else {
                  updateGame(game.id, (g) => ({ ...g, shortcut: !g.shortcut }))
                  toast(game.shortcut ? "Desktop shortcut removed" : "Desktop shortcut created", {
                    description: `crucible-${game.id}.desktop`,
                    tone: game.shortcut ? "danger" : "success",
                  })
                }
              }}
            />
          </div>
        </div>

        <div className="sticky top-0 z-10 border-b border-border bg-background/90 px-3 backdrop-blur-xl sm:px-6">
          <div role="tablist" aria-label="Game settings" className="flex gap-1 overflow-x-auto scrollbar-thin">
            {tabs.map((t) => {
              const active = tab === t.id
              return (
                <button
                  key={t.id}
                  type="button"
                  role="tab"
                  id={`tab-${t.id}`}
                  aria-selected={active}
                  aria-controls={`panel-${t.id}`}
                  onClick={() => setTab(t.id)}
                  className={cn(
                    "relative flex h-12 shrink-0 items-center gap-2 px-3 text-sm font-medium whitespace-nowrap transition-colors outline-none focus-visible:text-foreground",
                    active ? "text-foreground" : "text-muted-foreground hover:text-foreground",
                  )}
                >
                  <t.icon className={cn("size-4", active && "text-primary")} />
                  {t.label}
                  {active && <span className="absolute inset-x-2 bottom-0 h-0.5 rounded-full bg-primary" />}
                </button>
              )
            })}
          </div>
        </div>

        <div role="tabpanel" id={`panel-${tab}`} aria-labelledby={`tab-${tab}`} className="px-4 py-6 sm:px-8 sm:py-8">
          {tab === "overview" && <OverviewTab game={game} />}
          {tab === "launch" && <LaunchTab game={game} />}
          {tab === "graphics" && <GraphicsTab game={game} />}
          {tab === "advanced" && <AdvancedTab game={game} />}
          {tab === "data" && <DataTab game={game} />}
        </div>
      </div>
      <LogsDialog game={game} open={logsOpen} onClose={() => setLogsOpen(false)} />
    </>
  )
}

function ToolbarBtn({
  icon: Icon,
  label,
  onClick,
  active,
}: {
  icon: typeof Info
  label: string
  onClick: () => void
  active?: boolean
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "flex h-10 items-center gap-2 rounded-lg px-3 text-sm font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring",
        active ? "bg-primary/12 text-primary hover:bg-primary/20" : "text-muted-foreground hover:bg-accent hover:text-foreground",
      )}
    >
      <Icon className="size-4" />
      <span className="hidden md:inline">{label}</span>
      <span className="sr-only md:hidden">{label}</span>
    </button>
  )
}
