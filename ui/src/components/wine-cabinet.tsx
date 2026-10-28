import { Check, Download, HardDrive, Loader2, Trash2, Wine, X } from "lucide-react"
import { useState } from "react"
import { releases, sourceMeta } from "@/lib/data"
import type { Runner, RunnerSource } from "@/lib/types"
import { cn } from "@/lib/utils"
import { useCruce } from "./store"
import { Btn, Pill, Segmented } from "./ui"

export function WineCabinet() {
  const { runners, installed, games } = useCruce()
  const [source, setSource] = useState<RunnerSource>("ge")
  const list = runners.filter((r) => r.source === source)
  const installedList = runners.filter((r) => r.installed || installed.includes(r.id))
  const installedCount = installedList.length
  const totalSizeBytes = installedList.reduce((acc, r) => acc + (r.sizeBytes || 0), 0)
  const diskUsageStr = totalSizeBytes > 0 ? `${(totalSizeBytes / (1024 * 1024 * 1024)).toFixed(1)} GB` : `${(installedCount * 0.45).toFixed(1)} GB`

  return (
    <div className="mx-auto flex w-full max-w-5xl flex-col gap-8 px-5 py-8 sm:px-8">
      <header className="flex flex-col gap-6 md:flex-row md:items-end md:justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight text-balance">Wine Cabinet</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Download and manage Proton runners. Installed to{" "}
            <code className="font-mono text-foreground/80">compatibilitytools.d</code>.
          </p>
        </div>
        <div className="flex gap-3">
          <Stat icon={<Wine />} label="Installed" value={String(installedCount)} />
          <Stat icon={<HardDrive />} label="Disk usage" value={diskUsageStr} />
        </div>
      </header>

      <Segmented<RunnerSource>
        label="Runner source"
        value={source}
        onChange={setSource}
        className="self-start"
        options={(Object.keys(sourceMeta) as RunnerSource[]).map((s) => ({
          value: s,
          label: sourceMeta[s].label,
          count: runners.filter((r) => r.source === s && (r.installed || installed.includes(r.id))).length,
        }))}
      />

      <section aria-label={`${sourceMeta[source].label} releases`} className="overflow-hidden rounded-2xl border border-border bg-card/70">
        <div className="flex items-center justify-between border-b border-border px-5 py-3 text-xs text-muted-foreground">
          <span className="font-mono">github.com/{sourceMeta[source].repo}</span>
          <span>{list.length} releases</span>
        </div>
        {list.length === 0 ? (
          <div className="flex flex-col items-center justify-center p-8 text-center text-sm text-muted-foreground">
            <Wine className="mb-2 size-8 opacity-40" />
            <p>No releases found</p>
          </div>
        ) : (
          <ul className="divide-y divide-border">
            {list.map((r, i) => (
              <RunnerRow key={r.id} runner={r} latest={i === 0 && !r.installed} usedBy={games.filter((g) => g.runner === r.name).length} />
            ))}
          </ul>
        )}
      </section>
    </div>
  )
}

function Stat({ icon, label, value }: { icon: React.ReactNode; label: string; value: string }) {
  return (
    <div className="flex items-center gap-3 rounded-xl border border-border bg-card/70 px-4 py-2.5">
      <span className="text-primary [&_svg]:size-4">{icon}</span>
      <div>
        <p className="text-[11px] font-semibold tracking-widest text-muted-foreground uppercase">{label}</p>
        <p className="font-mono text-sm font-medium">{value}</p>
      </div>
    </div>
  )
}

function RunnerRow({ runner, latest, usedBy }: { runner: Runner; latest: boolean; usedBy: number }) {
  const { installed, download, startDownload, cancelDownload, removeRunner, toast } = useCruce()
  const isInstalled = Boolean(runner.installed || installed.includes(runner.id))
  const isDownloading = download?.runnerId === runner.id
  const busy = download !== null && !isDownloading

  return (
    <li className="relative flex flex-col gap-3 px-5 py-4 sm:flex-row sm:items-center sm:justify-between">
      {isDownloading && (
        <div
          aria-hidden
          className="absolute inset-y-0 left-0 bg-primary/[0.07] transition-[width] duration-200"
          style={{ width: `${download.progress}%` }}
        />
      )}
      <div className="relative flex min-w-0 items-center gap-4">
        <span
          className={cn(
            "flex size-10 shrink-0 items-center justify-center rounded-xl border",
            isInstalled ? "border-primary/30 bg-primary/10 text-primary" : "border-border bg-secondary text-muted-foreground",
          )}
        >
          {isInstalled ? <Check className="size-4" /> : <Wine className="size-4" />}
        </span>
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <p className="truncate font-mono text-sm font-medium">{runner.name}</p>
            {latest && <Pill tone="primary">Latest</Pill>}
            {usedBy > 0 && (
              <Pill tone="neutral">
                {usedBy} {usedBy === 1 ? "game" : "games"}
              </Pill>
            )}
          </div>
          <p className="mt-0.5 text-xs text-muted-foreground">
            {runner.date} · {runner.size}
          </p>
        </div>
      </div>

      <div className="relative flex items-center gap-2 sm:justify-end">
        {isDownloading ? (
          <>
            <span className="flex items-center gap-2 font-mono text-xs text-primary tabular-nums" aria-live="polite">
              <Loader2 className="size-3.5 animate-spin" />
              {Math.round(download.progress)}%
            </span>
            <Btn variant="ghost" size="sm" onClick={cancelDownload} aria-label={`Cancel ${runner.name} download`}>
              <X />
              Cancel
            </Btn>
          </>
        ) : isInstalled ? (
          <Btn
            variant="outline-danger"
            size="sm"
            onClick={() => {
              if (usedBy > 0) {
                toast("Runner in use", { description: `${usedBy} game(s) depend on ${runner.name}`, tone: "danger" })
                return
              }
              removeRunner(runner.id)
            }}
          >
            <Trash2 />
            Remove
          </Btn>
        ) : (
          <Btn variant={latest ? "primary" : "secondary"} size="sm" disabled={busy} onClick={() => startDownload(runner.id)}>
            <Download />
            Install
          </Btn>
        )}
      </div>
    </li>
  )
}
