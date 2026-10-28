import { FolderOpen, Loader2, RotateCcw, Save, Trash2, TriangleAlert, Upload } from "lucide-react"
import { useEffect, useState } from "react"
import type { BackupInfo, Game } from "@/lib/types"
import { useCruce } from "../store"
import { Btn, SectionCard } from "../ui"
import { backupGameSaves, getBackupInfo, isTauri, openFolder, resetPrefix, restoreGameSaves } from "@/lib/tauri"

export function DataTab({ game }: { game: Game }) {
  const { toast, updateGame, removeGame } = useCruce()
  const [confirm, setConfirm] = useState<"reset" | "delete" | null>(null)
  const [backupInfo, setBackupInfo] = useState<BackupInfo | null>(null)
  const [backingUp, setBackingUp] = useState(false)
  const [restoring, setRestoring] = useState(false)

  useEffect(() => {
    if (!isTauri()) return
    getBackupInfo(game.id)
      .then((info) => setBackupInfo(info))
      .catch((e) => console.error("Failed to get backup info:", e))
  }, [game.id])

  const handleBackup = async () => {
    if (!isTauri()) {
      toast("Native runtime required", { description: "Save backups require the native desktop runtime", tone: "info" })
      return
    }
    setBackingUp(true)
    try {
      const info = await backupGameSaves(game.id)
      setBackupInfo(info)
      toast("Saves backed up", {
        description: `${game.title} · ${info?.size_formatted ?? "Archived"}`,
      })
    } catch (e) {
      toast("Backup failed", { description: String(e), tone: "danger" })
    } finally {
      setBackingUp(false)
    }
  }

  const handleRestore = async () => {
    if (!isTauri()) {
      toast("Native runtime required", { description: "Save restore requires the native desktop runtime", tone: "info" })
      return
    }
    setRestoring(true)
    try {
      await restoreGameSaves(game.id)
      toast("Saves restored", {
        description: `Restored to ${game.title}'s prefix`,
      })
    } catch (e) {
      toast("Restore failed", { description: String(e), tone: "danger" })
    } finally {
      setRestoring(false)
    }
  }

  return (
    <div className="flex flex-col gap-6">
      <SectionCard title="Save data" description="Back up and restore saves from the prefix's user folders." icon={<Save />}>
        <div className="flex flex-col gap-4">
          <div className="flex items-center justify-between gap-4 rounded-xl border border-border bg-background/30 px-4 py-3">
            <div className="min-w-0">
              <p className="text-sm font-medium">Last backup</p>
              <p className="truncate font-mono text-xs text-muted-foreground">
                {backupInfo ? backupInfo.file_path : "No backup archive found"}
              </p>
            </div>
            {backupInfo ? (
              <div className="text-right shrink-0">
                <span className="block font-mono text-xs text-muted-foreground">{backupInfo.size_formatted}</span>
                <span className="block text-[10px] text-muted-foreground/70">{backupInfo.timestamp}</span>
              </div>
            ) : (
              <span className="shrink-0 font-mono text-xs text-muted-foreground/60">—</span>
            )}
          </div>
          <div className="flex flex-wrap gap-2">
            <Btn onClick={handleBackup} disabled={backingUp}>
              {backingUp ? <Loader2 className="animate-spin" /> : <Save />}
              Backup Saves
            </Btn>
            <Btn onClick={handleRestore} disabled={restoring || (!backupInfo && isTauri())}>
              {restoring ? <Loader2 className="animate-spin" /> : <Upload />}
              Restore Saves
            </Btn>
            <Btn
              variant="ghost"
              onClick={() => {
                if (isTauri()) {
                  openFolder(game.prefix)
                    .then(() => toast("Opened prefix folder", { description: game.prefix, tone: "info" }))
                    .catch((e) => toast("Failed to open prefix", { description: String(e), tone: "danger" }))
                  return
                }
                toast("Opened prefix folder", { description: game.prefix, tone: "info" })
              }}
            >
              <FolderOpen />
              Open prefix
            </Btn>
          </div>
        </div>
      </SectionCard>

      <SectionCard
        title="Danger zone"
        description="These actions cannot be undone."
        icon={<TriangleAlert />}
        tone="danger"
      >
        <div className="flex flex-col divide-y divide-destructive/15">
          <DangerRow
            title="Reset prefix"
            description="Deletes and recreates the Wine prefix. Installed redistributables and saves inside it are lost."
            confirming={confirm === "reset"}
            onAsk={() => setConfirm("reset")}
            onCancel={() => setConfirm(null)}
            onConfirm={() => {
              if (isTauri()) {
                resetPrefix(game.id).catch((e) => console.error("Reset prefix error:", e))
              }
              updateGame(game.id, (g) => ({ ...g, prefixSize: null }))
              setConfirm(null)
              toast("Prefix reset", { description: "A fresh prefix will be created on next launch", tone: "danger" })
            }}
            icon={<RotateCcw />}
            action="Reset Prefix"
          />
          <DangerRow
            title="Remove from library"
            description="Removes the game and its settings from Cruce. Game files stay on disk."
            confirming={confirm === "delete"}
            onAsk={() => setConfirm("delete")}
            onCancel={() => setConfirm(null)}
            onConfirm={() => removeGame(game.id)}
            icon={<Trash2 />}
            action="Delete Game"
          />
        </div>
      </SectionCard>
    </div>
  )
}

function DangerRow({
  title,
  description,
  confirming,
  onAsk,
  onCancel,
  onConfirm,
  icon,
  action,
}: {
  title: string
  description: string
  confirming: boolean
  onAsk: () => void
  onCancel: () => void
  onConfirm: () => void
  icon: React.ReactNode
  action: string
}) {
  return (
    <div className="flex flex-col gap-3 py-4 first:pt-0 last:pb-0 sm:flex-row sm:items-center sm:justify-between">
      <div className="max-w-lg">
        <p className="text-sm font-medium">{title}</p>
        <p className="mt-0.5 text-xs leading-relaxed text-muted-foreground">{description}</p>
      </div>
      {confirming ? (
        <div className="flex gap-2">
          <Btn variant="ghost" size="sm" onClick={onCancel}>
            Cancel
          </Btn>
          <Btn variant="danger" size="sm" onClick={onConfirm} autoFocus>
            {icon}
            Confirm
          </Btn>
        </div>
      ) : (
        <Btn variant="outline-danger" size="sm" onClick={onAsk}>
          {icon}
          {action}
        </Btn>
      )}
    </div>
  )
}
