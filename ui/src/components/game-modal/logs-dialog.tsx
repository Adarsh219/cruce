import { Copy, FolderOpen, ScrollText, Trash2, X } from "lucide-react"
import { useEffect, useState } from "react"
import type { Game } from "@/lib/types"
import { cn } from "@/lib/utils"
import { Dialog } from "../dialog"
import { useCruce } from "../store"
import { Btn } from "../ui"
import { clearGameLogs, getGameLogs, isTauri, openGameLogDir } from "@/lib/tauri"

export function LogsDialog({ game, open, onClose }: { game: Game; open: boolean; onClose: () => void }) {
  const { toast } = useCruce()
  const [realLines, setRealLines] = useState<[string, string][] | null>(null)

  const fetchLogs = () => {
    if (isTauri()) {
      getGameLogs(game.id)
        .then((l) => {
          setRealLines(l || [])
        })
        .catch(console.error)
    }
  }

  useEffect(() => {
    if (open) {
      fetchLogs()
    }
  }, [open, game.id])

  const handleClear = async () => {
    if (!isTauri()) return
    try {
      await clearGameLogs(game.id)
      setRealLines([])
      toast("Logs cleared", { tone: "info" })
    } catch (e) {
      toast("Failed to clear logs", { description: String(e), tone: "danger" })
    }
  }

  const handleOpenDir = async () => {
    if (!isTauri()) return
    try {
      await openGameLogDir(game.id)
    } catch (e) {
      toast("Failed to open log folder", { description: String(e), tone: "danger" })
    }
  }

  const lines = realLines ?? []
  const color: Record<string, string> = {
    info: "text-primary",
    debug: "text-muted-foreground",
    warn: "text-warning",
    error: "text-destructive font-medium",
  }

  return (
    <Dialog open={open} onClose={onClose} label={`${game.title} logs`} className="max-w-3xl" z="z-[60]">
      <header className="flex items-center justify-between border-b border-border px-5 py-3.5">
        <div>
          <h2 className="text-sm font-semibold">Logs · {game.title}</h2>
          <p className="font-mono text-[11px] text-muted-foreground">~/.local/share/crucible-launcher/logs/games/{game.id}/</p>
        </div>
        <div className="flex items-center gap-1">
          <Btn variant="ghost" size="sm" onClick={handleOpenDir} aria-label="Open log folder">
            <FolderOpen />
            Open Folder
          </Btn>
          <Btn variant="ghost" size="sm" onClick={handleClear} aria-label="Clear logs">
            <Trash2 />
            Clear
          </Btn>
          <Btn
            variant="ghost"
            size="sm"
            onClick={() => {
              if (typeof navigator !== "undefined" && navigator.clipboard) {
                const text = lines.map(([lvl, msg]) => `[${lvl}] ${msg}`).join("\n")
                navigator.clipboard.writeText(text).catch(console.error)
              }
              toast("Log copied to clipboard", { tone: "info" })
            }}
          >
            <Copy />
            Copy
          </Btn>
          <Btn variant="ghost" size="icon" onClick={onClose} aria-label="Close logs">
            <X />
          </Btn>
        </div>
      </header>
      {lines.length === 0 ? (
        <div className="flex flex-col items-center justify-center p-14 text-center text-muted-foreground">
          <ScrollText className="mb-2 size-8 opacity-40" />
          <p className="text-sm font-medium">No session logs recorded yet</p>
          <p className="mt-1 font-mono text-xs text-muted-foreground/70">Launch {game.title} to stream execution output</p>
        </div>
      ) : (
        <pre className="max-h-[60dvh] overflow-auto bg-chrome p-5 font-mono text-xs leading-6 scrollbar-thin">
          {lines.map(([lvl, msg], i) => (
            <div key={i} className="flex gap-3">
              <span className="text-muted-foreground/50 tabular-nums">{String(i + 1).padStart(2, "0")}</span>
              <span className={cn("w-11 shrink-0 uppercase", color[lvl] || "text-foreground")}>{lvl}</span>
              <span className="text-foreground/85">{msg}</span>
            </div>
          ))}
        </pre>
      )}
    </Dialog>
  )
}
