import { Gamepad2 } from "lucide-react"

interface GamepadHintsProps {
  connected: boolean
  gamepadName: string | null
  active: boolean
}

export function GamepadHints({ connected, gamepadName, active }: GamepadHintsProps) {
  if (!connected) return null

  return (
    <div className="flex h-7 shrink-0 items-center justify-between border-t border-border/60 bg-chrome/90 px-4 text-[11px] font-mono text-muted-foreground select-none backdrop-blur-sm">
      <div className="flex items-center gap-2">
        <Gamepad2 className="size-3.5 text-primary animate-pulse" />
        <span className="truncate max-w-[200px] text-foreground/80">{gamepadName || "Gamepad"}</span>
        <span className="size-1 rounded-full bg-success" />
      </div>

      <div className="flex items-center gap-3">
        <span className="inline-flex items-center gap-1">
          <kbd className="rounded border border-primary/40 bg-primary/20 px-1.5 py-0.2 text-[10px] font-bold text-primary">A</kbd>
          <span>Select</span>
        </span>
        <span className="inline-flex items-center gap-1">
          <kbd className="rounded border border-blue-500/40 bg-blue-500/20 px-1.5 py-0.2 text-[10px] font-bold text-blue-400">X</kbd>
          <span>Play/Stop</span>
        </span>
        <span className="inline-flex items-center gap-1">
          <kbd className="rounded border border-yellow-500/40 bg-yellow-500/20 px-1.5 py-0.2 text-[10px] font-bold text-yellow-400">Y</kbd>
          <span>Config</span>
        </span>
        <span className="inline-flex items-center gap-1">
          <kbd className="rounded border border-border bg-muted px-1.5 py-0.2 text-[10px] font-bold text-muted-foreground">LB/RB</kbd>
          <span>Tabs</span>
        </span>
        <span className="inline-flex items-center gap-1">
          <kbd className="rounded border border-destructive/40 bg-destructive/20 px-1.5 py-0.2 text-[10px] font-bold text-destructive">B</kbd>
          <span>Back</span>
        </span>
      </div>
    </div>
  )
}
