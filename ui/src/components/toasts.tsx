import { CheckCircle2, Info, Rocket, Trash2, X } from "lucide-react"
import { cn } from "@/lib/utils"
import type { ToastTone } from "@/lib/types"
import { useCruce } from "./store"

const toneMap: Record<ToastTone, { icon: typeof Info; cls: string }> = {
  success: { icon: CheckCircle2, cls: "text-success bg-success/12 border-success/25" },
  info: { icon: Info, cls: "text-primary bg-primary/12 border-primary/25" },
  danger: { icon: Trash2, cls: "text-destructive bg-destructive/12 border-destructive/25" },
  launch: { icon: Rocket, cls: "text-primary bg-primary/12 border-primary/25" },
}

export function Toasts() {
  const { toasts, dismissToast } = useCruce()
  return (
    <div
      aria-live="polite"
      className="pointer-events-none fixed right-4 bottom-4 left-4 z-[80] flex flex-col items-end gap-2 sm:left-auto sm:w-96"
    >
      {toasts.map((t) => {
        const { icon: Icon, cls } = toneMap[t.tone]
        return (
          <div
            key={t.id}
            role="status"
            className="pointer-events-auto flex w-full items-start gap-3 rounded-xl border border-border bg-popover/95 p-3.5 shadow-2xl shadow-black/50 backdrop-blur-xl animate-in fade-in slide-in-from-bottom-3 duration-300"
          >
            <span className={cn("flex size-8 shrink-0 items-center justify-center rounded-lg border", cls)}>
              <Icon className="size-4" />
            </span>
            <div className="min-w-0 flex-1 pt-0.5">
              <p className="text-sm font-semibold text-foreground">{t.title}</p>
              {t.description && <p className="mt-0.5 truncate text-xs text-muted-foreground">{t.description}</p>}
            </div>
            <button
              type="button"
              onClick={() => dismissToast(t.id)}
              aria-label="Dismiss notification"
              className="rounded-md p-1 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
            >
              <X className="size-3.5" />
            </button>
          </div>
        )
      })}
    </div>
  )
}
