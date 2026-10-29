import { FileCode2, Gamepad2, Plus, Wine } from "lucide-react"
import { useCruce } from "../store"
import { Btn } from "../ui"

export function EmptyLibrary() {
  const { setAddDialogOpen } = useCruce()
  return (
    <div className="flex min-h-full items-center justify-center px-6 py-16">
      <div className="flex max-w-md flex-col items-center text-center">
        <div className="relative mb-8 flex items-center justify-center">
          <div className="absolute size-40 rounded-full bg-primary/15 blur-3xl" />
          <div className="relative flex items-end gap-3">
            <span className="flex size-14 -rotate-12 items-center justify-center rounded-2xl border border-border bg-card text-muted-foreground">
              <FileCode2 className="size-6" />
            </span>
            <span className="flex size-20 items-center justify-center rounded-3xl border border-primary/40 bg-primary/10 text-primary shadow-[0_0_40px_-8px_var(--primary)]">
              <Gamepad2 className="size-9" />
            </span>
            <span className="flex size-14 rotate-12 items-center justify-center rounded-2xl border border-border bg-card text-muted-foreground">
              <Wine className="size-6" />
            </span>
          </div>
        </div>
        <h1 className="text-2xl font-semibold tracking-tight text-balance">Your library is empty</h1>
        <p className="mt-2 text-sm leading-relaxed text-muted-foreground text-pretty">
          Point Cruce at any Windows <code className="font-mono text-foreground/80">.exe</code> and it will create an
          isolated prefix, fetch artwork and launch it through umu-run.
        </p>
        <div className="mt-8 flex flex-col gap-2 sm:flex-row">
          <Btn variant="primary" size="lg" onClick={() => setAddDialogOpen(true)}>
            <Plus />
            Add your first game
          </Btn>
        </div>
      </div>
    </div>
  )
}
