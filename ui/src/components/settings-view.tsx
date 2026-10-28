import { Bug, Check, Cpu, Palette, RotateCcw, SlidersHorizontal } from "lucide-react"
import type { FontId, LogLevel, ThemeId } from "@/lib/types"
import { cn } from "@/lib/utils"
import { useCruce } from "./store"
import { Btn, LabeledInput, PathInput, SectionCard, Segmented, Select, ToggleRow } from "./ui"
import { browseDirectory, isTauri } from "@/lib/tauri"

const themes: { id: ThemeId; name: string; swatch: [string, string, string] }[] = [
  { id: "crucible", name: "Crucible", swatch: ["#121214", "#1c1c20", "#f59e0b"] },
  { id: "midnight", name: "Midnight", swatch: ["#0b0f1a", "#141a2b", "#60a5fa"] },
  { id: "catppuccin", name: "Catppuccin", swatch: ["#1e1e2e", "#313244", "#cba6f7"] },
  { id: "nord", name: "Nord", swatch: ["#2e3440", "#3b4252", "#88c0d0"] },
]

export function SettingsView() {
  const { settings: s, updateSettings: set, installedRunnerNames, toast, restoreDemo } = useCruce()

  return (
    <div className="mx-auto flex w-full max-w-4xl flex-col gap-6 px-5 py-8 sm:px-8">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
        <p className="mt-1 text-sm text-muted-foreground">Changes are saved to ~/.config/cruce/config.toml automatically.</p>
      </header>

      <SectionCard title="General" icon={<SlidersHorizontal />}>
        <div className="grid gap-3">
          <ToggleRow label="Minimize to tray" description="Keep Cruce running in the system tray when closed." checked={s.minimizeToTray} onChange={(minimizeToTray) => set({ minimizeToTray })} />
          <ToggleRow label="Remember window geometry" description="Restore size and position on next launch." checked={s.rememberGeometry} onChange={(rememberGeometry) => set({ rememberGeometry })} />
          <ToggleRow label="Auto-update umu-launcher" description="Check for umu-run updates on startup." checked={s.autoUpdateUmu} onChange={(autoUpdateUmu) => set({ autoUpdateUmu })} />
        </div>
      </SectionCard>

      <SectionCard title="Appearance" icon={<Palette />}>
        <div className="flex flex-col gap-6">
          <fieldset>
            <legend className="mb-3 text-sm font-medium text-foreground/90">Theme</legend>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
              {themes.map((t) => {
                const active = s.theme === t.id
                return (
                  <button
                    key={t.id}
                    type="button"
                    aria-pressed={active}
                    onClick={() => set({ theme: t.id })}
                    className={cn(
                      "group relative overflow-hidden rounded-xl border p-1.5 text-left transition-all outline-none focus-visible:ring-2 focus-visible:ring-ring",
                      active ? "border-primary ring-1 ring-primary" : "border-border hover:border-foreground/25",
                    )}
                  >
                    <div className="flex h-16 overflow-hidden rounded-lg" style={{ background: t.swatch[0] }}>
                      <div className="w-1/3" style={{ background: t.swatch[1] }} />
                      <div className="flex flex-1 flex-col justify-end gap-1 p-2">
                        <div className="h-1.5 w-3/4 rounded-full bg-white/15" />
                        <div className="h-1.5 w-1/3 rounded-full" style={{ background: t.swatch[2] }} />
                      </div>
                    </div>
                    <div className="flex items-center justify-between px-1.5 pt-2 pb-1">
                      <span className="text-sm font-medium">{t.name}</span>
                      {active && <Check className="size-3.5 text-primary" />}
                    </div>
                  </button>
                )
              })}
            </div>
          </fieldset>
          <div className="flex flex-col gap-2">
            <span className="text-sm font-medium text-foreground/90">Interface font</span>
            <Segmented<FontId>
              label="Interface font"
              value={s.font}
              onChange={(font) => set({ font })}
              className="self-start"
              options={[
                { value: "geist", label: "Geist" },
                { value: "inter", label: "Inter" },
                { value: "mono", label: "Mono" },
              ]}
            />
          </div>
        </div>
      </SectionCard>

      <SectionCard title="Proton defaults" description="Applied to newly added games." icon={<Cpu />}>
        <div className="flex flex-col gap-5">
          <PathInput
            label="Proton directory"
            value={s.protonDir}
            onChange={(protonDir) => set({ protonDir })}
            onBrowse={() => {
              if (isTauri()) {
                browseDirectory().then((chosen) => {
                  if (chosen) set({ protonDir: chosen })
                })
              } else {
                toast("Select Proton directory", { tone: "info" })
              }
            }}
          />
          <div className="grid gap-5 md:grid-cols-2">
            <Select
              label="Default runner"
              value={s.defaultRunner}
              onChange={(defaultRunner) => set({ defaultRunner })}
              options={(installedRunnerNames.includes(s.defaultRunner) ? installedRunnerNames : [s.defaultRunner, ...installedRunnerNames]).map((r) => ({ value: r, label: r }))}
            />
            <LabeledInput label="Default launch arguments" value={s.defaultArgs} onChange={(defaultArgs) => set({ defaultArgs })} placeholder="-skipintro" mono />
            <LabeledInput label="Default DLL overrides" value={s.defaultDll} onChange={(defaultDll) => set({ defaultDll })} placeholder="dxgi=n,b" mono />
            <LabeledInput label="Default wrapper" value={s.defaultWrapper} onChange={(defaultWrapper) => set({ defaultWrapper })} placeholder="obs-gamecapture" mono />
          </div>
          <div className="grid gap-3 md:grid-cols-3">
            <ToggleRow label="GameMode" checked={s.defaultGamemode} onChange={(defaultGamemode) => set({ defaultGamemode })} />
            <ToggleRow label="MangoHud" checked={s.defaultMangohud} onChange={(defaultMangohud) => set({ defaultMangohud })} />
            <ToggleRow label="Gamescope" checked={s.defaultGamescope} onChange={(defaultGamescope) => set({ defaultGamescope })} />
          </div>
        </div>
      </SectionCard>

      <SectionCard title="Diagnostics" icon={<Bug />}>
        <div className="flex flex-col gap-6">
          <div className="flex flex-col gap-2">
            <span className="text-sm font-medium text-foreground/90">Log level</span>
            <Segmented<LogLevel>
              label="Log level"
              value={s.logLevel}
              onChange={(logLevel) => set({ logLevel })}
              className="self-start"
              options={(["error", "warn", "info", "debug", "trace"] as LogLevel[]).map((l) => ({ value: l, label: l[0].toUpperCase() + l.slice(1) }))}
            />
          </div>
          {!isTauri() && (
            <div className="flex flex-col gap-3 rounded-xl border border-border bg-background/30 p-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p className="text-sm font-medium">Restore demo library</p>
                <p className="text-xs text-muted-foreground">Bring back the sample games for exploring Cruce.</p>
              </div>
              <Btn size="sm" onClick={restoreDemo}>
                <RotateCcw />
                Restore
              </Btn>
            </div>
          )}
        </div>
      </SectionCard>

      <p className="pb-4 text-center font-mono text-xs text-muted-foreground">Cruce v0.3.0 · umu-launcher 1.2.9 · Tauri v2 / Rust</p>
    </div>
  )
}
