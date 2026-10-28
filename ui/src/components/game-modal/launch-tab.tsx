import { Braces, Plus, Terminal, Trash2 } from "lucide-react"
import { useRef } from "react"
import type { Game, LaunchConfig } from "@/lib/types"
import { useCruce } from "../store"
import { Btn, LabeledInput, SectionCard, TextInput } from "../ui"

export function LaunchTab({ game }: { game: Game }) {
  const { updateGame } = useCruce()
  const envId = useRef(0)
  const set = (patch: Partial<LaunchConfig>) => updateGame(game.id, (g) => ({ ...g, launch: { ...g.launch, ...patch } }))
  const { launch } = game

  const preview = [
    ...launch.env.filter((e) => e.key).map((e) => `${e.key}=${e.value}`),
    launch.dllOverrides && `WINEDLLOVERRIDES="${launch.dllOverrides}"`,
    launch.wrapper,
    "umu-run",
    `"${game.exe.split("/").pop()}"`,
    launch.args,
  ]
    .filter(Boolean)
    .join(" ")

  return (
    <div className="flex flex-col gap-6">
      <SectionCard title="Command line" description="Arguments, overrides and wrappers applied on launch." icon={<Terminal />}>
        <div className="grid gap-5 md:grid-cols-2">
          <LabeledInput
            label="Launch arguments"
            value={launch.args}
            onChange={(args) => set({ args })}
            placeholder="-dx12 -windowed"
            mono
            className="md:col-span-2"
          />
          <LabeledInput
            label="DLL overrides"
            value={launch.dllOverrides}
            onChange={(dllOverrides) => set({ dllOverrides })}
            placeholder="dxgi=n,b;d3d11=n"
            mono
            hint={
              <>
                Exported as <code className="font-mono">WINEDLLOVERRIDES</code>
              </>
            }
          />
          <LabeledInput
            label="Wrapper command"
            value={launch.wrapper}
            onChange={(wrapper) => set({ wrapper })}
            placeholder="obs-gamecapture"
            mono
            hint="Prepended before umu-run."
          />
        </div>
      </SectionCard>

      <SectionCard
        title="Environment variables"
        description="Per-game variables injected into the Wine environment."
        icon={<Braces />}
        action={
          <Btn
            size="sm"
            onClick={() => set({ env: [...launch.env, { id: `new-${game.id}-${++envId.current}`, key: "", value: "" }] })}
          >
            <Plus />
            Add
          </Btn>
        }
      >
        {launch.env.length === 0 ? (
          <p className="rounded-xl border border-dashed border-border px-4 py-6 text-center text-sm text-muted-foreground">
            No custom variables. Add one like <code className="font-mono text-foreground/80">DXVK_HUD=fps</code>.
          </p>
        ) : (
          <ul className="flex flex-col gap-2">
            {launch.env.map((e) => (
              <li key={e.id} className="flex gap-2">
                <TextInput
                  aria-label="Variable name"
                  mono
                  value={e.key}
                  placeholder="KEY"
                  onChange={(ev) =>
                    set({ env: launch.env.map((x) => (x.id === e.id ? { ...x, key: ev.target.value.toUpperCase().replace(/\s/g, "_") } : x)) })
                  }
                  className="w-2/5"
                />
                <TextInput
                  aria-label="Variable value"
                  mono
                  value={e.value}
                  placeholder="value"
                  onChange={(ev) => set({ env: launch.env.map((x) => (x.id === e.id ? { ...x, value: ev.target.value } : x)) })}
                />
                <Btn
                  variant="ghost"
                  size="icon"
                  aria-label={`Remove ${e.key || "variable"}`}
                  onClick={() => set({ env: launch.env.filter((x) => x.id !== e.id) })}
                >
                  <Trash2 />
                </Btn>
              </li>
            ))}
          </ul>
        )}
      </SectionCard>

      <div className="rounded-2xl border border-border bg-chrome p-4">
        <p className="mb-2 text-[11px] font-semibold tracking-widest text-muted-foreground uppercase">Resolved command</p>
        <pre className="overflow-x-auto font-mono text-xs leading-relaxed whitespace-pre-wrap text-foreground/85 scrollbar-thin">
          <span className="text-primary">$ </span>
          {preview}
        </pre>
      </div>
    </div>
  )
}
