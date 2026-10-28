import { cn } from "@/lib/utils"

export function CruceLogo({ className }: { className?: string }) {
  return (
    <span
      aria-hidden="true"
      className={cn(
        "relative flex size-7 items-center justify-center rounded-lg bg-primary shadow-[0_0_24px_-4px_var(--primary)]",
        className,
      )}
    >
      <svg viewBox="0 0 24 24" className="size-[62%] text-primary-foreground" fill="none">
        <path d="M5 6h14l-2.5 11a2 2 0 0 1-2 1.5h-5a2 2 0 0 1-2-1.5L5 6Z" stroke="currentColor" strokeWidth="2.2" strokeLinejoin="round" />
        <path d="M9 11.5h6" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" />
      </svg>
    </span>
  )
}
