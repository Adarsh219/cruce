import { useEffect, useRef } from "react"
import { Clock, Loader2, Play, Settings2, Square } from "lucide-react"
import { formatLastPlayed, formatPlaytime } from "@/lib/format"
import type { Game } from "@/lib/types"
import { cn } from "@/lib/utils"
import { CoverArt, GameIcon } from "../game-art"
import { useCruce } from "../store"
import { Pill } from "../ui"

export function GameCard({ game, priority, isFocused }: { game: Game; priority?: boolean; isFocused?: boolean }) {
  const { togglePlay, setOpenGameId } = useCruce()
  const isRunning = game.status === "running"
  const isLaunching = game.status === "launching"
  const btnRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (isFocused && btnRef.current) {
      btnRef.current.scrollIntoView({ behavior: "smooth", block: "nearest" })
    }
  }, [isFocused])

  return (
    <article className="group flex flex-col">
      <button
        ref={btnRef}
        type="button"
        onClick={() => setOpenGameId(game.id)}
        aria-label={`Open ${game.title} details`}
        className={cn(
          "relative aspect-[2/3] overflow-hidden rounded-xl border bg-card outline-none transition-all duration-300 focus-visible:ring-2 focus-visible:ring-ring group-hover:-translate-y-1 group-hover:shadow-2xl group-hover:shadow-black/60",
          isRunning ? "border-success/50 shadow-[0_0_0_1px_var(--success),0_20px_40px_-20px_var(--success)]" : "border-border group-hover:border-foreground/20",
          isFocused && "ring-3 ring-primary ring-offset-2 ring-offset-background -translate-y-1 shadow-2xl shadow-primary/30",
        )}
      >
        <CoverArt game={game} sizes="(min-width: 1536px) 14vw, (min-width: 1024px) 18vw, (min-width: 640px) 30vw, 45vw" priority={priority} />
        <div className="pointer-events-none absolute inset-0 bg-[linear-gradient(to_bottom,rgba(0,0,0,0.55)_0%,transparent_22%,transparent_60%,rgba(0,0,0,0.75)_100%)]" />

        <GameIcon game={game} className="absolute top-2.5 left-2.5" />

        <div className="absolute top-2.5 right-2.5">
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

        <div className="absolute inset-x-2.5 bottom-2.5 flex items-center gap-1.5 font-mono text-[11px] text-white/85">
          <Clock className="size-3" />
          {formatPlaytime(game.playtimeMin)}
        </div>
      </button>

      <div className="mt-3 flex min-w-0 flex-col gap-0.5 px-0.5">
        <h3 className="truncate text-sm font-semibold text-foreground">{game.title}</h3>
        <p className="truncate font-mono text-[11px] text-muted-foreground">{game.runner}</p>
        <p className="text-xs text-muted-foreground">
          Last played <span className="text-foreground/80">{formatLastPlayed(game.lastPlayedDaysAgo)}</span>
        </p>
      </div>

      <div className="mt-3 flex gap-2">
        <button
          type="button"
          onClick={() => togglePlay(game.id)}
          disabled={isLaunching}
          className={cn(
            "flex h-9 flex-1 items-center justify-center gap-2 rounded-lg text-sm font-semibold transition-all outline-none focus-visible:ring-2 focus-visible:ring-ring active:scale-[0.97] disabled:opacity-80",
            isRunning
              ? "bg-destructive/15 text-destructive ring-1 ring-destructive/30 hover:bg-destructive hover:text-destructive-foreground"
              : "bg-primary text-primary-foreground hover:brightness-110",
          )}
        >
          {isLaunching ? (
            <Loader2 className="size-4 animate-spin" />
          ) : isRunning ? (
            <Square className="size-3.5 fill-current" />
          ) : (
            <Play className="size-4 fill-current" />
          )}
          {isLaunching ? "Starting" : isRunning ? "Stop" : "Play"}
        </button>
        <button
          type="button"
          onClick={() => setOpenGameId(game.id)}
          aria-label={`${game.title} settings`}
          className="flex size-9 items-center justify-center rounded-lg border border-border bg-secondary text-muted-foreground transition-colors outline-none hover:bg-accent hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
        >
          <Settings2 className="size-4" />
        </button>
      </div>
    </article>
  )
}
