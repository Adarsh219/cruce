import { useEffect, useRef } from "react"
import { createPortal } from "react-dom"
import { cn } from "@/lib/utils"

export function Dialog({
  open,
  onClose,
  label,
  children,
  className,
  z = "z-50",
}: {
  open: boolean
  onClose: () => void
  label: string
  children: React.ReactNode
  className?: string
  z?: string
}) {
  const panelRef = useRef<HTMLDivElement>(null)
  const onCloseRef = useRef(onClose)
  onCloseRef.current = onClose

  useEffect(() => {
    if (!open) return
    const prev = document.activeElement as HTMLElement | null
    panelRef.current?.focus()
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation()
        onCloseRef.current()
      }
    }
    panelRef.current?.addEventListener("keydown", onKey)
    const node = panelRef.current
    return () => {
      node?.removeEventListener("keydown", onKey)
      prev?.focus?.()
    }
  }, [open])

  if (!open) return null

  return createPortal(
    <div className={cn("fixed inset-0 flex items-center justify-center p-2 sm:p-6", z)}>
      <div
        aria-hidden="true"
        onClick={onClose}
        className="absolute inset-0 bg-black/70 backdrop-blur-sm animate-in fade-in duration-200"
      />
      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-label={label}
        tabIndex={-1}
        className={cn(
          "relative flex max-h-full w-full flex-col overflow-hidden rounded-2xl border border-border bg-background shadow-2xl shadow-black/70 outline-none animate-in fade-in zoom-in-95 duration-200",
          className,
        )}
      >
        {children}
      </div>
    </div>,
    document.body,
  )
}
