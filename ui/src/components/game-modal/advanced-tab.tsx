import { Box, FolderArchive, Layers, TriangleAlert } from "lucide-react"
import type { Game, WineConfig } from "@/lib/types"
import { useCruce } from "../store"
import { Btn, SectionCard, ToggleRow } from "../ui"
import { applyZipToGame, browseFile, isTauri } from "@/lib/tauri"

export function AdvancedTab({ game }: { game: Game }) {
  const { updateGame, toast } = useCruce()
  const w = game.wine
  const set = (patch: Partial<WineConfig>) => updateGame(game.id, (x) => ({ ...x, wine: { ...x.wine, ...patch } }))

  const handleApplyZip = async () => {
    if (isTauri()) {
      const chosen = await browseFile("Mod Archive (*.zip)", ["zip"])
      if (chosen) {
        try {
          const res = await applyZipToGame(game.id, chosen)
          if (res?.success) {
            updateGame(game.id, (g) => ({
              ...g,
              ...(res.exe ? { exe: res.exe } : {}),
              launch: {
                ...g.launch,
                dllOverrides: res.dlls.length
                  ? Array.from(new Set([...g.launch.dllOverrides.split(",").filter(Boolean), ...res.dlls.map((d) => `${d}=n,b`)])).join(",")
                  : g.launch.dllOverrides,
              },
            }))
            toast("Mod applied", {
              description: `Extracted ${res.dlls.length} DLL(s) into game directory`,
            })
          }
        } catch (e) {
          toast("Failed to apply mod", { description: String(e), tone: "danger" })
        }
      }
      return
    }
    toast("Select mod zip", { description: "Select a .zip archive to extract into install directory", tone: "info" })
  }

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-start gap-3 rounded-xl border border-warning/25 bg-warning/[0.07] px-4 py-3 text-sm">
        <TriangleAlert className="mt-0.5 size-4 shrink-0 text-warning" />
        <p className="text-muted-foreground">
          <span className="font-medium text-foreground">Defaults work for most games.</span> Only change these if a
          ProtonDB report or the game&apos;s logs suggest it.
        </p>
      </div>

      <SectionCard title="Mod & patch archive" description="Extract a mod .zip directly into the game directory and register DLL overrides." icon={<FolderArchive />}>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div className="text-xs text-muted-foreground">
            Automatically extracts files with path traversal checks, configures DLL overrides (e.g. DXVK, ReShade), and updates executable paths.
          </div>
          <Btn variant="secondary" size="sm" onClick={handleApplyZip} className="shrink-0">
            <FolderArchive />
            Apply Mod (.zip)
          </Btn>
        </div>
      </SectionCard>

      <SectionCard title="Graphics & display layer" description="How Wine talks to your GPU and compositor." icon={<Layers />}>
        <div className="grid gap-3 md:grid-cols-2">
          <ToggleRow label="NVAPI" description="Expose NVIDIA API for DLSS & Reflex." tag="PROTON_ENABLE_NVAPI=1" checked={w.nvapi} onChange={(nvapi) => set({ nvapi })} />
          <ToggleRow label="Native Wayland" description="Use winewayland instead of XWayland." tag="PROTON_ENABLE_WAYLAND=1" checked={w.wayland} onChange={(wayland) => set({ wayland })} />
          <ToggleRow label="HDR" description="Requires Wayland or Gamescope." tag="PROTON_ENABLE_HDR=1" checked={w.hdr} onChange={(hdr) => set({ hdr })} />
          <ToggleRow label="WineD3D" description="OpenGL fallback instead of DXVK." tag="PROTON_USE_WINED3D=1" checked={w.wined3d} onChange={(wined3d) => set({ wined3d })} />
        </div>
      </SectionCard>

      <SectionCard title="Runtime & sandbox" description="Prefix architecture, logging and container behaviour." icon={<Box />}>
        <div className="grid gap-3 md:grid-cols-2">
          <ToggleRow label="WoW64 mode" description="New-style 32-on-64 without multilib." tag="PROTON_USE_WOW64=1" checked={w.wow64} onChange={(wow64) => set({ wow64 })} />
          <ToggleRow label="Wine logging" description="Write steam-<appid>.log to $HOME." tag="PROTON_LOG=1" checked={w.log} onChange={(log) => set({ log })} />
          <ToggleRow label="Disable Steam client" description="Skip lsteamclient for non-Steam builds." tag="PROTON_NO_STEAMCLIENT=1" checked={w.noSteamClient} onChange={(noSteamClient) => set({ noSteamClient })} />
          <ToggleRow label="Skip runtime update" description="Don't refresh the Steam Linux Runtime." tag="UMU_RUNTIME_UPDATE=0" checked={w.skipRuntimeUpdate} onChange={(skipRuntimeUpdate) => set({ skipRuntimeUpdate })} />
          <ToggleRow label="Bubblewrap lock" description="Prevent concurrent pressure-vessel containers." tag="UMU_CONTAINER_NSENTER=1" checked={w.bwrapLock} onChange={(bwrapLock) => set({ bwrapLock })} />
        </div>
      </SectionCard>
    </div>
  )
}
