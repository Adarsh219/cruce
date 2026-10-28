import { ChevronDown, FolderOpen } from "lucide-react"
import { useId } from "react"
import { cn } from "@/lib/utils"

type BtnVariant = "primary" | "secondary" | "ghost" | "danger" | "outline-danger"
type BtnSize = "sm" | "md" | "lg" | "icon"

const btnVariants: Record<BtnVariant, string> = {
  primary:
    "bg-primary text-primary-foreground hover:brightness-110 shadow-[0_0_0_1px_color-mix(in_oklch,var(--primary)_60%,transparent),0_8px_24px_-8px_color-mix(in_oklch,var(--primary)_60%,transparent)]",
  secondary: "bg-secondary text-secondary-foreground border border-border hover:bg-accent",
  ghost: "text-muted-foreground hover:text-foreground hover:bg-accent",
  danger: "bg-destructive text-destructive-foreground hover:brightness-110",
  "outline-danger":
    "border border-destructive/40 text-destructive hover:bg-destructive/10 hover:border-destructive/70",
}

const btnSizes: Record<BtnSize, string> = {
  sm: "h-8 px-3 text-xs gap-1.5 rounded-md",
  md: "h-10 px-4 text-sm gap-2 rounded-lg",
  lg: "h-12 px-6 text-base gap-2.5 rounded-xl",
  icon: "size-10 rounded-lg",
}

export function Btn({
  variant = "secondary",
  size = "md",
  className,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & { variant?: BtnVariant; size?: BtnSize }) {
  return (
    <button
      type="button"
      className={cn(
        "inline-flex shrink-0 items-center justify-center font-medium whitespace-nowrap transition-all outline-none focus-visible:ring-2 focus-visible:ring-ring active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-4 [&_svg]:shrink-0",
        btnVariants[variant],
        btnSizes[size],
        className,
      )}
      {...props}
    />
  )
}

export function Kbd({ children, className }: { children: React.ReactNode; className?: string }) {
  return (
    <kbd
      className={cn(
        "inline-flex h-5 min-w-5 items-center justify-center rounded border border-border bg-background/60 px-1.5 font-mono text-[10px] font-medium text-muted-foreground",
        className,
      )}
    >
      {children}
    </kbd>
  )
}

export function Switch({
  checked,
  onChange,
  label,
  disabled,
}: {
  checked: boolean
  onChange: (v: boolean) => void
  label: string
  disabled?: boolean
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={cn(
        "relative inline-flex h-6 w-11 shrink-0 items-center rounded-full border transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-40",
        checked ? "border-primary/60 bg-primary" : "border-border bg-muted",
      )}
    >
      <span
        className={cn(
          "inline-block size-4.5 rounded-full shadow-sm transition-transform",
          checked ? "translate-x-5.5 bg-primary-foreground" : "translate-x-0.5 bg-foreground/70",
        )}
      />
    </button>
  )
}

export function SectionCard({
  title,
  description,
  icon,
  action,
  children,
  className,
  tone = "default",
}: {
  title: string
  description?: string
  icon?: React.ReactNode
  action?: React.ReactNode
  children: React.ReactNode
  className?: string
  tone?: "default" | "danger"
}) {
  return (
    <section
      className={cn(
        "rounded-2xl border bg-card/70",
        tone === "danger" ? "border-destructive/30 bg-destructive/[0.04]" : "border-border",
        className,
      )}
    >
      <header className="flex items-start justify-between gap-4 px-5 pt-5 pb-4 sm:px-6">
        <div className="flex items-start gap-3">
          {icon && (
            <span
              className={cn(
                "mt-0.5 flex size-8 items-center justify-center rounded-lg border [&_svg]:size-4",
                tone === "danger"
                  ? "border-destructive/30 bg-destructive/10 text-destructive"
                  : "border-border bg-secondary text-primary",
              )}
            >
              {icon}
            </span>
          )}
          <div>
            <h3 className="text-[15px] font-semibold text-foreground">{title}</h3>
            {description && <p className="mt-0.5 text-sm text-muted-foreground text-pretty">{description}</p>}
          </div>
        </div>
        {action}
      </header>
      <div className="px-5 pb-5 sm:px-6 sm:pb-6">{children}</div>
    </section>
  )
}

export function Field({
  label,
  hint,
  children,
  htmlFor,
  className,
}: {
  label: string
  hint?: React.ReactNode
  children: React.ReactNode
  htmlFor?: string
  className?: string
}) {
  return (
    <div className={cn("flex flex-col gap-2", className)}>
      <label htmlFor={htmlFor} className="text-sm font-medium text-foreground/90">
        {label}
      </label>
      {children}
      {hint && <p className="text-xs leading-relaxed text-muted-foreground">{hint}</p>}
    </div>
  )
}

const inputCls =
  "h-10 w-full min-w-0 rounded-lg border border-input bg-background/60 px-3 text-sm text-foreground placeholder:text-muted-foreground/60 transition-colors outline-none hover:border-foreground/20 focus:border-primary/60 focus:ring-2 focus:ring-ring/40 disabled:opacity-50"

export function TextInput({ className, mono, ...props }: React.InputHTMLAttributes<HTMLInputElement> & { mono?: boolean }) {
  return <input className={cn(inputCls, mono && "font-mono text-[13px]", className)} {...props} />
}

export function LabeledInput({
  label,
  hint,
  value,
  onChange,
  placeholder,
  mono,
  className,
}: {
  label: string
  hint?: React.ReactNode
  value: string
  onChange: (v: string) => void
  placeholder?: string
  mono?: boolean
  className?: string
}) {
  const id = useId()
  return (
    <Field label={label} hint={hint} htmlFor={id} className={className}>
      <TextInput id={id} value={value} onChange={(e) => onChange(e.target.value)} placeholder={placeholder} mono={mono} />
    </Field>
  )
}

export function PathInput({
  label,
  hint,
  value,
  onChange,
  onBrowse,
  placeholder,
}: {
  label: string
  hint?: React.ReactNode
  value: string
  onChange: (v: string) => void
  onBrowse: () => void
  placeholder?: string
}) {
  const id = useId()
  return (
    <Field label={label} hint={hint} htmlFor={id}>
      <div className="flex gap-2">
        <TextInput id={id} mono value={value} onChange={(e) => onChange(e.target.value)} placeholder={placeholder} />
        <Btn onClick={onBrowse} aria-label={`Browse for ${label}`}>
          <FolderOpen />
          <span className="hidden sm:inline">Browse</span>
        </Btn>
      </div>
    </Field>
  )
}

export function Select<T extends string>({
  label,
  hint,
  value,
  onChange,
  options,
  className,
}: {
  label: string
  hint?: React.ReactNode
  value: T
  onChange: (v: T) => void
  options: { value: T; label: string }[]
  className?: string
}) {
  const id = useId()
  return (
    <Field label={label} hint={hint} htmlFor={id} className={className}>
      <div className="relative">
        <select
          id={id}
          value={value}
          onChange={(e) => onChange(e.target.value as T)}
          className={cn(inputCls, "cursor-pointer appearance-none pr-9 [&>option]:bg-popover")}
        >
          {options.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
        <ChevronDown className="pointer-events-none absolute top-1/2 right-3 size-4 -translate-y-1/2 text-muted-foreground" />
      </div>
    </Field>
  )
}

export function ToggleRow({
  label,
  description,
  tag,
  checked,
  onChange,
  disabled,
  badge,
}: {
  label: string
  description?: string
  tag?: string
  checked: boolean
  onChange: (v: boolean) => void
  disabled?: boolean
  badge?: React.ReactNode
}) {
  return (
    <div
      className={cn(
        "flex items-center justify-between gap-6 rounded-xl border px-4 py-3.5 transition-colors",
        checked ? "border-primary/25 bg-primary/[0.05]" : "border-border bg-background/30",
        disabled && "opacity-50",
      )}
    >
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-sm font-medium text-foreground">{label}</span>
          {badge}
        </div>
        {description && <p className="mt-0.5 text-xs leading-relaxed text-muted-foreground text-pretty">{description}</p>}
        {tag && <code className="mt-1.5 inline-block font-mono text-[11px] text-muted-foreground/80">{tag}</code>}
      </div>
      <Switch checked={checked} onChange={onChange} label={label} disabled={disabled} />
    </div>
  )
}

export function Segmented<T extends string>({
  value,
  onChange,
  options,
  label,
  className,
}: {
  value: T
  onChange: (v: T) => void
  options: { value: T; label: string; icon?: React.ReactNode; count?: number }[]
  label: string
  className?: string
}) {
  return (
    <div role="radiogroup" aria-label={label} className={cn("inline-flex rounded-xl border border-border bg-background/50 p-1", className)}>
      {options.map((o) => {
        const active = o.value === value
        return (
          <button
            key={o.value}
            type="button"
            role="radio"
            aria-checked={active}
            onClick={() => onChange(o.value)}
            className={cn(
              "inline-flex h-8 items-center gap-2 rounded-lg px-3 text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:ring-2 focus-visible:ring-ring [&_svg]:size-4",
              active ? "bg-secondary text-foreground shadow-sm ring-1 ring-border" : "text-muted-foreground hover:text-foreground",
            )}
          >
            {o.icon}
            {o.label}
            {o.count !== undefined && (
              <span className={cn("rounded-md px-1.5 text-xs tabular-nums", active ? "bg-primary/15 text-primary" : "bg-muted")}>
                {o.count}
              </span>
            )}
          </button>
        )
      })}
    </div>
  )
}

export function Pill({
  tone = "neutral",
  children,
  className,
  pulse,
}: {
  tone?: "neutral" | "success" | "warning" | "primary" | "danger"
  children: React.ReactNode
  className?: string
  pulse?: boolean
}) {
  const tones = {
    neutral: "bg-black/55 text-foreground/80 border-white/10",
    success: "bg-success/15 text-success border-success/30",
    warning: "bg-warning/15 text-warning border-warning/30",
    primary: "bg-primary/15 text-primary border-primary/30",
    danger: "bg-destructive/15 text-destructive border-destructive/30",
  }
  const dot = {
    neutral: "bg-foreground/50",
    success: "bg-success",
    warning: "bg-warning",
    primary: "bg-primary",
    danger: "bg-destructive",
  }
  return (
    <span
      className={cn(
        "inline-flex h-6 items-center gap-1.5 rounded-full border px-2.5 text-[11px] font-semibold tracking-wide uppercase backdrop-blur-md",
        tones[tone],
        className,
      )}
    >
      <span className="relative flex size-1.5">
        {pulse && <span className={cn("absolute inset-0 animate-ping rounded-full opacity-75", dot[tone])} />}
        <span className={cn("relative size-1.5 rounded-full", dot[tone])} />
      </span>
      {children}
    </span>
  )
}
