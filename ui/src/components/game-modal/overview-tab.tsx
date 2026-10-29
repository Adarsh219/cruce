import { Cpu, FolderTree, Wrench } from "lucide-react"
import type { Game } from "@/lib/types"
import { useCruce } from "../store"
import { Btn, PathInput, SectionCard, Select } from "../ui"
import { browseDirectory, browseFile, isTauri, openWineTool } from "@/lib/tauri"

export function OverviewTab({ game }: { game: Game }) {
  const { updateGame, toast, installedRunnerNames, setOpenGameId, setView } = useCruce()
  const set = (patch: Partial<Game>) => updateGame(game.id, (g) => ({ ...g, ...patch }))
  const browse = (what: string) => {
    if (isTauri()) {
      if (what === "executable") {
        browseFile("Windows Executable", ["exe", "bat", "cmd"]).then((chosen) => {
          if (chosen) set({ exe: chosen })
        })
      } else if (what === "install directory") {
        browseDirectory().then((chosen) => {
          if (chosen) set({ installDir: chosen })
        })
      } else if (what === "prefix") {
        browseDirectory().then((chosen) => {
          if (chosen) set({ prefix: chosen })
        })
      }
    } else {
      toast(`Select ${what}`, { description: "Native file chooser opened via xdg-desktop-portal", tone: "info" })
    }
  }

  const runners = installedRunnerNames.includes(game.runner) ? installedRunnerNames : [game.runner, ...installedRunnerNames]

  return (
    <div className="grid gap-6 lg:grid-cols-[1fr_320px]">
      <SectionCard title="Paths" description="Where Cruce finds your game and its Wine prefix." icon={<FolderTree />}>
        <div className="flex flex-col gap-5">
          <PathInput label="Executable" value={game.exe} onChange={(exe) => set({ exe })} onBrowse={() => browse("executable")} />
          <PathInput
            label="Install directory"
            value={game.installDir}
            onChange={(installDir) => set({ installDir })}
            onBrowse={() => browse("install directory")}
            hint="Used as the working directory when launching."
          />
          <PathInput
            label="Wine prefix"
            value={game.prefix}
            onChange={(prefix) => set({ prefix })}
            onBrowse={() => browse("prefix")}
            hint="Each game gets an isolated prefix by default."
          />
        </div>
      </SectionCard>

      <div className="flex flex-col gap-6">
        <SectionCard title="Runner" description="Proton build used by umu-run." icon={<Cpu />}>
          <Select
            label="Proton version"
            value={game.runner}
            onChange={(runner) => {
              set({ runner })
              toast("Runner updated", { description: runner })
            }}
            options={runners.map((r) => ({ value: r, label: r }))}
          />
          {!installedRunnerNames.includes(game.runner) && (
            <p className="mt-3 rounded-lg border border-warning/30 bg-warning/10 px-3 py-2 text-xs text-warning">
              This runner is not installed.{" "}
              <button
                type="button"
                className="font-semibold underline underline-offset-2"
                onClick={() => {
                  setOpenGameId(null)
                  setView("cabinet")
                }}
              >
                Open Wine Cabinet
              </button>
            </p>
          )}
        </SectionCard>

        <SectionCard title="Wine Power Tools" description="Prefix utilities and configuration." icon={<Wrench />}>
          <div className="grid grid-cols-2 gap-2">
            <Btn
              variant="secondary"
              onClick={() => {
                if (isTauri()) {
                  openWineTool(game.id, "winecfg")
                    .then(() => toast("Opening Winecfg", { description: "Wine configuration utility", tone: "info" }))
                    .catch((e) => toast("Failed to open Winecfg", { description: String(e), tone: "danger" }))
                  return
                }
                toast("Opening Winecfg", { description: "Wine configuration utility", tone: "info" })
              }}
            >
              Winecfg
            </Btn>
            <Btn
              variant="secondary"
              onClick={() => {
                if (isTauri()) {
                  openWineTool(game.id, "regedit")
                    .then(() => toast("Opening Regedit", { description: "Wine registry editor", tone: "info" }))
                    .catch((e) => toast("Failed to open Regedit", { description: String(e), tone: "danger" }))
                  return
                }
                toast("Opening Regedit", { description: "Wine registry editor", tone: "info" })
              }}
            >
              Regedit
            </Btn>
            <Btn
              variant="secondary"
              onClick={() => {
                if (isTauri()) {
                  openWineTool(game.id, "taskmgr")
                    .then(() => toast("Opening Taskmgr", { description: "Wine task manager", tone: "info" }))
                    .catch((e) => toast("Failed to open Taskmgr", { description: String(e), tone: "danger" }))
                  return
                }
                toast("Opening Taskmgr", { description: "Wine task manager", tone: "info" })
              }}
            >
              Taskmgr
            </Btn>
            <Btn
              variant="secondary"
              onClick={() => {
                if (isTauri()) {
                  openWineTool(game.id, "winetricks")
                    .then(() => toast("Opening Winetricks", { description: "Component and font installer", tone: "info" }))
                    .catch((e) => toast("Failed to open Winetricks", { description: String(e), tone: "danger" }))
                  return
                }
                toast("Opening Winetricks", { description: "Component and font installer", tone: "info" })
              }}
            >
              Winetricks
            </Btn>
          </div>
        </SectionCard>

        <dl className="grid grid-cols-2 gap-px overflow-hidden rounded-2xl border border-border bg-border">
          {[
            ["Prefix size", game.prefixSize ?? "Not created"],
            ["Architecture", game.wine.wow64 ? "WoW64" : "win64"],
            ["Gamescope", game.graphics.gamescope ? "On" : "Off"],
            ["Shortcut", game.shortcut ? "Yes" : "No"],
          ].map(([k, v]) => (
            <div key={k} className="bg-card p-4">
              <dt className="text-[11px] font-semibold tracking-widest text-muted-foreground uppercase">{k}</dt>
              <dd className="mt-1 font-mono text-sm">{v}</dd>
            </div>
          ))}
        </dl>
      </div>
    </div>
  )
}
