import { FileCode2, FolderOpen, Gamepad2, X } from "lucide-react"
import { useEffect, useState } from "react"
import { mockExecutables } from "@/lib/data"
import { Dialog } from "../dialog"
import { useCruce } from "../store"
import { Btn, LabeledInput, PathInput, Select } from "../ui"
import { browseDirectory, browseFile, isTauri } from "@/lib/tauri"

export function AddGameDialog() {
  const { addDialogOpen, setAddDialogOpen, addGame, installedRunnerNames, settings } = useCruce()
  const [exePath, setExePath] = useState("")
  const [installDir, setInstallDir] = useState("")
  const [title, setTitle] = useState("")
  const [runner, setRunner] = useState(settings.defaultRunner)

  useEffect(() => {
    if (settings.defaultRunner) {
      setRunner(settings.defaultRunner)
    }
  }, [settings.defaultRunner])

  const close = () => {
    setAddDialogOpen(false)
    setExePath("")
    setInstallDir("")
    setTitle("")
  }

  const handleBrowseExe = async () => {
    if (isTauri()) {
      const chosen = await browseFile("Windows Executable (*.exe)", ["exe", "bat", "cmd"])
      if (chosen) {
        setExePath(chosen)
        const parts = chosen.split("/").filter(Boolean)
        const fileName = parts.pop() || ""
        const cleanName = fileName.replace(/\.exe$/i, "")
        const parentDir = "/" + parts.join("/")
        setInstallDir(parentDir)

        if (["launcher", "game", "start", "shipping"].includes(cleanName.toLowerCase()) && parts.length > 0) {
          setTitle(parts[parts.length - 1])
        } else {
          setTitle(cleanName)
        }
      }
    }
  }

  const handleBrowseInstallDir = async () => {
    if (isTauri()) {
      const chosen = await browseDirectory()
      if (chosen) {
        setInstallDir(chosen)
      }
    }
  }

  const selectMock = (e: (typeof mockExecutables)[0]) => {
    setExePath(`/home/deck/Games/${e.folder}/${e.exe}`)
    setInstallDir(`/home/deck/Games/${e.folder}`)
    setTitle(e.title)
  }

  const submit = () => {
    const trimmedExe = exePath.trim()
    if (!trimmedExe) return

    const derivedDir = installDir.trim() || trimmedExe.substring(0, trimmedExe.lastIndexOf("/"))
    const derivedTitle = title.trim() || trimmedExe.split("/").pop()?.replace(/\.exe$/i, "") || "New Game"
    const digest = trimmedExe.split("").reduce((acc, c) => acc + c.charCodeAt(0), 0)

    addGame({
      title: derivedTitle,
      exe: trimmedExe,
      installDir: derivedDir,
      hue: digest % 360,
      runner: runner || settings.defaultRunner,
    })
    close()
  }

  const runnerOptions = installedRunnerNames.length ? installedRunnerNames : [settings.defaultRunner]

  return (
    <Dialog open={addDialogOpen} onClose={close} label="Add game" className="max-w-xl">
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div>
          <h2 className="text-base font-semibold">Add Game</h2>
          <p className="text-xs text-muted-foreground">Select a Windows executable to create an isolated Proton prefix</p>
        </div>
        <Btn variant="ghost" size="icon" onClick={close} aria-label="Close">
          <X />
        </Btn>
      </header>

      <div className="flex flex-col gap-5 p-6">
        <div>
          <PathInput
            label="Executable (.exe)"
            value={exePath}
            onChange={(val) => {
              setExePath(val)
              if (!title && val) {
                const fn = val.split("/").pop()?.replace(/\.exe$/i, "")
                if (fn) setTitle(fn)
              }
              if (!installDir && val.includes("/")) {
                setInstallDir(val.substring(0, val.lastIndexOf("/")))
              }
            }}
            onBrowse={handleBrowseExe}
            placeholder="/path/to/game.exe"
          />
        </div>

        <LabeledInput
          label="Game title"
          value={title}
          onChange={setTitle}
          placeholder="e.g. Cyberpunk 2077"
        />

        <PathInput
          label="Working / install directory"
          value={installDir}
          onChange={setInstallDir}
          onBrowse={handleBrowseInstallDir}
          placeholder="/path/to/game"
        />

        <Select
          label="Proton runner"
          value={runner}
          onChange={setRunner}
          options={runnerOptions.map((r) => ({ value: r, label: r }))}
        />

        {!isTauri() && (
          <div className="mt-2 rounded-xl border border-dashed border-border bg-card/40 p-3">
            <p className="mb-2 text-xs font-medium text-muted-foreground">Web Preview Presets:</p>
            <div className="flex flex-wrap gap-2">
              {mockExecutables.map((m) => (
                <button
                  key={m.exe}
                  type="button"
                  role="option"
                  onClick={() => selectMock(m)}
                  className="rounded-lg border border-border bg-secondary/60 px-2.5 py-1 text-xs text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
                >
                  {m.title}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      <footer className="flex items-center justify-end gap-2 border-t border-border px-6 py-4">
        <Btn variant="ghost" onClick={close}>
          Cancel
        </Btn>
        <Btn variant="primary" onClick={submit} disabled={!exePath.trim()}>
          <Gamepad2 />
          Add to Library
        </Btn>
      </footer>
    </Dialog>
  )
}
