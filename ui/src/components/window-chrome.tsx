import { Copy, Minus, Square, X } from "lucide-react"
import { CruceLogo } from "./logo"
import { useCruce } from "./store"
import { closeWindow, isTauri, minimizeWindow, toggleMaximizeWindow } from "@/lib/tauri"

export function WindowChrome({
  maximized,
  onMinimize,
  onMaximize,
  onClose,
}: {
  maximized: boolean
  onMinimize: () => void
  onMaximize: () => void
  onClose: () => void
}) {
  const { games } = useCruce()
  const running = games.filter((g) => g.status === "running").length

  const handleMinimize = () => {
    if (isTauri()) minimizeWindow()
    else onMinimize()
  }

  const handleMaximize = () => {
    if (isTauri()) toggleMaximizeWindow()
    else onMaximize()
  }

  const handleClose = () => {
    if (isTauri()) closeWindow()
    else onClose()
  }

  return (
    <header
      data-tauri-drag-region
      onDoubleClick={handleMaximize}
      className="flex h-11 shrink-0 items-center justify-between border-b border-border bg-chrome pl-4 select-none"
    >
      <div className="flex items-center gap-2.5">
        <CruceLogo className="size-6" />
        <span className="text-sm font-semibold tracking-tight">Cruce</span>
        <span className="hidden rounded-md border border-border px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground sm:inline">
          v0.9.2
        </span>
      </div>

      <div className="hidden items-center gap-2 font-mono text-[11px] text-muted-foreground md:flex">
        <span className="size-1.5 rounded-full bg-success" />
        umu-run 1.2.9
        <span className="text-border">|</span>
        {running > 0 ? `${running} game running` : "Wayland · Mesa 26.1"}
      </div>

      <div className="flex h-full items-stretch" onDoubleClick={(e) => e.stopPropagation()}>
        <button
          type="button"
          onClick={handleMinimize}
          aria-label="Minimize"
          className="flex w-12 items-center justify-center text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        >
          <Minus className="size-4" />
        </button>
        <button
          type="button"
          onClick={handleMaximize}
          aria-label={maximized ? "Restore window" : "Maximize"}
          className="flex w-12 items-center justify-center text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        >
          {maximized ? <Copy className="size-3.5 -scale-x-100" /> : <Square className="size-3.5" />}
        </button>
        <button
          type="button"
          onClick={handleClose}
          aria-label="Close window"
          className="flex w-12 items-center justify-center text-muted-foreground transition-colors hover:bg-destructive hover:text-destructive-foreground"
        >
          <X className="size-4" />
        </button>
      </div>
    </header>
  )
}
