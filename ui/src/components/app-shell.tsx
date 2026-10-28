import { Power } from "lucide-react"
import { useEffect, useState } from "react"
import { cn } from "@/lib/utils"
import { CruceLogo } from "./logo"
import { GameModal } from "./game-modal/game-modal"
import { AddGameDialog } from "./library/add-game-dialog"
import { LibraryView } from "./library/library-view"
import { SettingsView } from "./settings-view"
import { Sidebar } from "./sidebar"
import { CruceProvider, useCruce } from "./store"
import { Toasts } from "./toasts"
import { Btn } from "./ui"
import { WindowChrome } from "./window-chrome"
import { WineCabinet } from "./wine-cabinet"
import { GamepadHints } from "./gamepad-hints"

export function CruceApp() {
  return (
    <CruceProvider>
      <Shell />
    </CruceProvider>
  )
}

function Shell() {
  const { view, settings, toast, gamepad } = useCruce()
  const [maximized, setMaximized] = useState(true)
  const [closed, setClosed] = useState(false)

  useEffect(() => {
    document.documentElement.dataset.theme = settings.theme
    document.documentElement.dataset.font = settings.font
  }, [settings.theme, settings.font])

  return (
    <div data-theme={settings.theme} data-font={settings.font} className="min-h-dvh bg-chrome font-sans text-foreground">
      {closed ? (
        <div className="flex min-h-dvh flex-col items-center justify-center gap-6 p-6 text-center">
          <CruceLogo className="size-14" />
          <div>
            <h1 className="text-xl font-semibold">Cruce is closed</h1>
            <p className="mt-1 text-sm text-muted-foreground">
              {settings.minimizeToTray ? "Still running in the system tray." : "All windows were closed."}
            </p>
          </div>
          <Btn variant="primary" onClick={() => setClosed(false)}>
            <Power />
            Reopen Cruce
          </Btn>
        </div>
      ) : (
        <div className={cn("flex h-dvh flex-col transition-all duration-300", !maximized && "p-2 sm:p-6 lg:p-10")}>
          <div
            className={cn(
              "flex min-h-0 flex-1 flex-col overflow-hidden bg-background",
              !maximized && "rounded-2xl border border-border shadow-2xl shadow-black/60",
            )}
          >
            <WindowChrome
              maximized={maximized}
              onMinimize={() =>
                toast(settings.minimizeToTray ? "Minimized to tray" : "Window minimized", {
                  description: "Cruce keeps watching running games",
                  tone: "info",
                })
              }
              onMaximize={() => setMaximized((m) => !m)}
              onClose={() => setClosed(true)}
            />
            <div className="flex min-h-0 flex-1 flex-col md:flex-row">
              <Sidebar />
              <main className="ember-glow relative min-h-0 flex-1 overflow-y-auto scrollbar-thin">
                {view === "library" && <LibraryView />}
                {view === "cabinet" && <WineCabinet />}
                {view === "settings" && <SettingsView />}
              </main>
            </div>
            <GamepadHints
              connected={gamepad.connected}
              gamepadName={gamepad.gamepadName}
              active={gamepad.gamepadActive}
            />
          </div>
        </div>
      )}
      <GameModal />
      <AddGameDialog />
      <Toasts />
    </div>
  )
}
