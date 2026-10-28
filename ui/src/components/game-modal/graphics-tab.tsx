import { Activity, ChevronDown, Maximize, Sparkles } from "lucide-react"
import type { GamescopeConfig, Game, GraphicsConfig, UpscaleMethod, WindowType } from "@/lib/types"
import { cn } from "@/lib/utils"
import { useCruce } from "../store"
import { LabeledInput, SectionCard, Select, Segmented, Switch, ToggleRow } from "../ui"

export function GraphicsTab({ game }: { game: Game }) {
  const { updateGame } = useCruce()
  const g = game.graphics
  const set = (patch: Partial<GraphicsConfig>) =>
    updateGame(game.id, (x) => ({ ...x, graphics: { ...x.graphics, ...patch } }))
  const setGs = (patch: Partial<GamescopeConfig>) =>
    updateGame(game.id, (x) => ({ ...x, graphics: { ...x.graphics, gs: { ...x.graphics.gs, ...patch } } }))

  return (
    <div className="flex flex-col gap-6">
      <SectionCard title="Performance" description="System-level tuning and overlays." icon={<Activity />}>
        <div className="grid gap-3 md:grid-cols-2">
          <ToggleRow
            label="GameMode"
            description="Feral GameMode CPU governor & scheduler tweaks."
            tag="gamemoderun"
            checked={g.gamemode}
            onChange={(gamemode) => set({ gamemode })}
          />
          <ToggleRow
            label="MangoHud"
            description="FPS, frametime and temperatures overlay."
            tag="MANGOHUD=1"
            checked={g.mangohud}
            onChange={(mangohud) => set({ mangohud })}
          />
        </div>
      </SectionCard>

      <section
        className={cn(
          "rounded-2xl border transition-colors",
          g.gamescope ? "border-primary/30 bg-primary/[0.03]" : "border-border bg-card/70",
        )}
      >
        <header className="flex items-start justify-between gap-4 p-5 sm:px-6">
          <div className="flex items-start gap-3">
            <span className="mt-0.5 flex size-8 items-center justify-center rounded-lg border border-border bg-secondary text-primary">
              <Maximize className="size-4" />
            </span>
            <div>
              <h3 className="text-[15px] font-semibold">Gamescope</h3>
              <p className="mt-0.5 text-sm text-muted-foreground">Micro-compositor for upscaling, frame limiting and HDR.</p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <ChevronDown className={cn("hidden size-4 text-muted-foreground transition-transform sm:block", g.gamescope && "rotate-180")} />
            <Switch checked={g.gamescope} onChange={(gamescope) => set({ gamescope })} label="Enable Gamescope" />
          </div>
        </header>
        <div
          className={cn(
            "grid transition-all duration-300 ease-out",
            g.gamescope ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0",
          )}
        >
          <div className="overflow-hidden" inert={!g.gamescope}>
            <div className="flex flex-col gap-6 border-t border-border px-5 py-6 sm:px-6">
              <div className="grid gap-6 md:grid-cols-2">
                <ResolutionPair
                  label="Render resolution"
                  w={g.gs.renderW}
                  h={g.gs.renderH}
                  onW={(renderW) => setGs({ renderW })}
                  onH={(renderH) => setGs({ renderH })}
                />
                <ResolutionPair
                  label="Output resolution"
                  w={g.gs.outW}
                  h={g.gs.outH}
                  onW={(outW) => setGs({ outW })}
                  onH={(outH) => setGs({ outH })}
                />
              </div>
              <div className="grid gap-6 md:grid-cols-3">
                <Select<UpscaleMethod>
                  label="Upscale method"
                  value={g.gs.method}
                  onChange={(method) => setGs({ method })}
                  options={[
                    { value: "fsr", label: "AMD FSR 1" },
                    { value: "nis", label: "NVIDIA NIS" },
                    { value: "integer", label: "Integer scaling" },
                    { value: "pixel", label: "Nearest (pixel)" },
                    { value: "stretch", label: "Stretch" },
                  ]}
                />
                <LabeledInput label="FPS limit" value={g.gs.fps} onChange={(fps) => setGs({ fps })} placeholder="Unlimited" mono />
                <LabeledInput
                  label="Unfocused FPS"
                  value={g.gs.fpsUnfocused}
                  onChange={(fpsUnfocused) => setGs({ fpsUnfocused })}
                  placeholder="30"
                  mono
                />
              </div>
              <div className="flex flex-col gap-4 md:flex-row md:items-end">
                <div className="flex flex-col gap-2">
                  <span className="text-sm font-medium text-foreground/90">Window type</span>
                  <Segmented<WindowType>
                    label="Window type"
                    value={g.gs.windowType}
                    onChange={(windowType) => setGs({ windowType })}
                    options={[
                      { value: "borderless", label: "Borderless" },
                      { value: "fullscreen", label: "Fullscreen" },
                    ]}
                  />
                </div>
                <div className="flex-1">
                  <ToggleRow
                    label="Grab cursor"
                    tag="--force-grab-cursor"
                    checked={g.gs.grabCursor}
                    onChange={(grabCursor) => setGs({ grabCursor })}
                  />
                </div>
              </div>
              <LabeledInput
                label="Extra arguments"
                value={g.gs.extra}
                onChange={(extra) => setGs({ extra })}
                placeholder="--hdr-enabled --adaptive-sync"
                mono
              />
            </div>
          </div>
        </div>
      </section>

      <SectionCard
        title="Upscalers & frame generation"
        description="Inject or upgrade vendor upscalers via Proton."
        icon={<Sparkles />}
      >
        <div className="grid gap-3 md:grid-cols-2">
          <ToggleRow label="NVIDIA DLSS" tag="PROTON_ENABLE_NGX_UPDATER=1" checked={g.dlss} onChange={(dlss) => set({ dlss })} />
          <ToggleRow label="AMD FSR 3" tag="PROTON_FSR3_UPGRADE=1" checked={g.fsr3} onChange={(fsr3) => set({ fsr3 })} />
          <ToggleRow
            label="AMD FSR 4"
            tag="PROTON_FSR4_UPGRADE=1"
            checked={g.fsr4}
            onChange={(fsr4) => set({ fsr4, fsr4rdna3: fsr4 ? g.fsr4rdna3 : false })}
          />
          <ToggleRow
            label="FSR 4 on RDNA 3"
            description="Emulated path for RX 7000 series."
            tag="PROTON_FSR4_RDNA3_UPGRADE=1"
            checked={g.fsr4rdna3}
            onChange={(fsr4rdna3) => set({ fsr4rdna3 })}
            disabled={!g.fsr4}
            badge={<span className="rounded bg-warning/15 px-1.5 py-0.5 text-[10px] font-semibold text-warning uppercase">Experimental</span>}
          />
          <ToggleRow label="Intel XeSS" tag="PROTON_XESS_UPGRADE=1" checked={g.xess} onChange={(xess) => set({ xess })} />
        </div>
      </SectionCard>
    </div>
  )
}

function ResolutionPair({
  label,
  w,
  h,
  onW,
  onH,
}: {
  label: string
  w: string
  h: string
  onW: (v: string) => void
  onH: (v: string) => void
}) {
  const clean = (v: string) => v.replace(/\D/g, "").slice(0, 5)
  return (
    <fieldset className="flex flex-col gap-2">
      <legend className="mb-2 text-sm font-medium text-foreground/90">{label}</legend>
      <div className="flex items-center gap-2">
        <input
          aria-label={`${label} width`}
          inputMode="numeric"
          value={w}
          onChange={(e) => onW(clean(e.target.value))}
          className="h-10 w-full rounded-lg border border-input bg-background/60 px-3 text-center font-mono text-sm outline-none focus:border-primary/60 focus:ring-2 focus:ring-ring/40"
        />
        <span className="text-muted-foreground">×</span>
        <input
          aria-label={`${label} height`}
          inputMode="numeric"
          value={h}
          onChange={(e) => onH(clean(e.target.value))}
          className="h-10 w-full rounded-lg border border-input bg-background/60 px-3 text-center font-mono text-sm outline-none focus:border-primary/60 focus:ring-2 focus:ring-ring/40"
        />
      </div>
    </fieldset>
  )
}
