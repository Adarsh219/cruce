import { Gamepad2, Settings2, Square, Wine } from "lucide-react"
import { formatPlaytime } from "@/lib/format"
import type { View } from "@/lib/types"
import { cn } from "@/lib/utils"
import { GameIcon } from "./game-art"
import { useCruce } from "./store"

export function Sidebar() {
  const { view, setView, games, installed, togglePlay, setOpenGameId, download } = useCruce()
  const active = games.find((g) => g.status !== "idle")

  const items: { id: View; label: string; icon: typeof Gamepad2; count?: number }[] = [
    { id: "library", label: "Library", icon: Gamepad2, count: games.length },
    { id: "cabinet", label: "Wine Cabinet", icon: Wine, count: installed.length },
    { id: "settings", label: "Settings", icon: Settings2 },
  ]

  return (
    <aside className="flex shrink-0 flex-col border-b border-border bg-chrome/60 md:w-64 md:border-r md:border-b-0">
      <nav aria-label="Primary" className="flex gap-1 overflow-x-auto p-2 md:flex-col md:p-3">
        <p className="hidden px-3 pt-3 pb-2 text-[11px] font-semibold tracking-widest text-muted-foreground/70 uppercase md:block">
          Navigate
        </p>
        {items.map((it) => {
          const isActive = view === it.id
          return (
            <button
              key={it.id}
              type="button"
              onClick={() => setView(it.id)}
              aria-current={isActive ? "page" : undefined}
              className={cn(
                "group relative flex h-10 shrink-0 items-center gap-3 rounded-lg px-3 text-sm font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring",
                isActive ? "bg-secondary text-foreground" : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
              )}
            >
              {isActive && <span className="absolute top-2 bottom-2 left-0 hidden w-0.5 rounded-full bg-primary md:block" />}
              <it.icon className={cn("size-4.5", isActive && "text-primary")} />
              {it.label}
              {it.id === "cabinet" && download && <span className="size-1.5 animate-pulse rounded-full bg-primary" />}
              {it.count !== undefined && (
                <span className="ml-auto hidden rounded-md bg-muted px-1.5 py-0.5 text-xs tabular-nums text-muted-foreground md:inline">
                  {it.count}
                </span>
              )}
            </button>
          )
        })}
      </nav>

      {active && (
        <div className="mt-auto hidden p-3 md:block">
          <div className="rounded-xl border border-primary/25 bg-primary/[0.06] p-3">
            <p className="text-[11px] font-semibold tracking-widest text-primary uppercase">
              {active.status === "launching" ? "Launching" : "Now Playing"}
            </p>
            <button
              type="button"
              onClick={() => setOpenGameId(active.id)}
              className="mt-2.5 flex w-full items-center gap-3 rounded-lg text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <GameIcon game={active} className="size-10" />
              <span className="min-w-0">
                <span className="block truncate text-sm font-semibold">{active.title}</span>
                <span className="block font-mono text-[11px] text-muted-foreground">{formatPlaytime(active.playtimeMin)} total</span>
              </span>
            </button>
            <button
              type="button"
              onClick={() => togglePlay(active.id)}
              className="mt-3 flex h-8 w-full items-center justify-center gap-2 rounded-md bg-destructive/15 text-xs font-semibold text-destructive transition-colors hover:bg-destructive hover:text-destructive-foreground"
            >
              <Square className="size-3 fill-current" />
              Stop
            </button>
          </div>
        </div>
      )}
    </aside>
  )
}
