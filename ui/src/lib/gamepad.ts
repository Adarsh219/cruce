import { useEffect, useRef, useState, useCallback } from "react"
import type { Game, View } from "./types"

export interface GamepadState {
  connected: boolean
  gamepadName: string | null
  gamepadActive: boolean
  focusedIndex: number
}

interface UseGamepadOptions {
  games: Game[]
  view: View
  setView: (v: View) => void
  openGameId: string | null
  setOpenGameId: (id: string | null) => void
  addDialogOpen: boolean
  setAddDialogOpen: (open: boolean) => void
  togglePlay: (id: string) => void
  toast: (title: string, opts?: { description?: string; tone?: "success" | "info" | "danger" | "launch" }) => void
}

export function useGamepadNavigation({
  games,
  view,
  setView,
  openGameId,
  setOpenGameId,
  addDialogOpen,
  setAddDialogOpen,
  togglePlay,
  toast,
}: UseGamepadOptions) {
  const [connected, setConnected] = useState(false)
  const [gamepadName, setGamepadName] = useState<string | null>(null)
  const [gamepadActive, setGamepadActive] = useState(false)
  const [focusedIndex, setFocusedIndex] = useState(0)

  const lastActionTime = useRef(0)
  const lastButtonState = useRef<Record<number, boolean>>({})

  // Views order for bumper switching
  const views: View[] = ["library", "cabinet", "settings"]

  const handleBumper = useCallback(
    (direction: -1 | 1) => {
      const currentIndex = views.indexOf(view)
      const nextIndex = (currentIndex + direction + views.length) % views.length
      setView(views[nextIndex])
    },
    [view, setView],
  )

  useEffect(() => {
    const onConnect = (e: GamepadEvent) => {
      setConnected(true)
      const name = e.gamepad.id.replace(/\s*\(.*\)$/, "")
      setGamepadName(name)
      setGamepadActive(true)
      toast("Gamepad Connected", {
        description: name || "Standard Gamepad / Steam Deck Controller",
        tone: "info",
      })
    }

    const onDisconnect = () => {
      const pads = navigator.getGamepads ? navigator.getGamepads() : []
      const anyPad = Array.from(pads).some((p) => p !== null && p.connected)
      if (!anyPad) {
        setConnected(false)
        setGamepadName(null)
        setGamepadActive(false)
        toast("Gamepad Disconnected", { tone: "info" })
      }
    }

    const onMouseMove = () => {
      if (gamepadActive) setGamepadActive(false)
    }

    window.addEventListener("gamepadconnected", onConnect)
    window.addEventListener("gamepaddisconnected", onDisconnect)
    window.addEventListener("mousemove", onMouseMove)

    // Initial check
    if (typeof navigator !== "undefined" && navigator.getGamepads) {
      const pads = navigator.getGamepads()
      for (const pad of pads) {
        if (pad && pad.connected) {
          setConnected(true)
          setGamepadName(pad.id.replace(/\s*\(.*\)$/, ""))
          break
        }
      }
    }

    return () => {
      window.removeEventListener("gamepadconnected", onConnect)
      window.removeEventListener("gamepaddisconnected", onDisconnect)
      window.removeEventListener("mousemove", onMouseMove)
    }
  }, [gamepadActive, toast])

  useEffect(() => {
    let animId: number

    const poll = () => {
      const pads = navigator.getGamepads ? navigator.getGamepads() : []
      const pad = pads[0] || Array.from(pads).find((p) => p !== null && p.connected)

      if (pad && pad.connected) {
        const now = performance.now()
        const isCooldown = now - lastActionTime.current < 170

        // Buttons
        const isPressed = (idx: number) => {
          const btn = pad.buttons[idx]
          if (!btn) return false
          return typeof btn === "object" ? btn.pressed : btn === 1.0
        }

        // Check if any controller input occurred
        let inputOccurred = false
        for (let i = 0; i < pad.buttons.length; i++) {
          if (isPressed(i)) {
            inputOccurred = true
            break
          }
        }
        for (let i = 0; i < pad.axes.length; i++) {
          if (Math.abs(pad.axes[i]) > 0.45) {
            inputOccurred = true
            break
          }
        }

        if (inputOccurred && !gamepadActive) {
          setGamepadActive(true)
        }

        // Just pressed edge trigger
        const justPressed = (idx: number) => {
          const curr = isPressed(idx)
          const prev = lastButtonState.current[idx] || false
          lastButtonState.current[idx] = curr
          return curr && !prev
        }

        // Navigation (D-Pad or Left Stick)
        const leftStickX = pad.axes[0] || 0
        const leftStickY = pad.axes[1] || 0

        const up = isPressed(12) || leftStickY < -0.48
        const down = isPressed(13) || leftStickY > 0.48
        const left = isPressed(14) || leftStickX < -0.48
        const right = isPressed(15) || leftStickX > 0.48

        if (!isCooldown) {
          if (view === "library" && games.length > 0) {
            if (right) {
              setFocusedIndex((prev) => Math.min(prev + 1, games.length - 1))
              lastActionTime.current = now
            } else if (left) {
              setFocusedIndex((prev) => Math.max(prev - 1, 0))
              lastActionTime.current = now
            } else if (down) {
              setFocusedIndex((prev) => Math.min(prev + 4, games.length - 1))
              lastActionTime.current = now
            } else if (up) {
              setFocusedIndex((prev) => Math.max(prev - 4, 0))
              lastActionTime.current = now
            }
          }
        }

        // Bumper view switching: LB (button 4), RB (button 5)
        if (justPressed(4)) {
          handleBumper(-1)
        } else if (justPressed(5)) {
          handleBumper(1)
        }

        // Button 1 (B): Back / Close dialogs
        if (justPressed(1)) {
          if (openGameId) {
            setOpenGameId(null)
          } else if (addDialogOpen) {
            setAddDialogOpen(false)
          }
        }

        // Button 0 (A): Primary Action (Open details or confirm)
        if (justPressed(0)) {
          if (!openGameId && !addDialogOpen && view === "library" && games[focusedIndex]) {
            setOpenGameId(games[focusedIndex].id)
          }
        }

        // Button 2 (X): Toggle Play/Stop on focused game
        if (justPressed(2)) {
          if (view === "library" && games[focusedIndex]) {
            togglePlay(games[focusedIndex].id)
          }
        }

        // Button 3 (Y): Configure / Open Game Modal
        if (justPressed(3)) {
          if (view === "library" && games[focusedIndex]) {
            setOpenGameId(games[focusedIndex].id)
          }
        }

        // Button 8 (Select/View): Toggle Add Game dialog
        if (justPressed(8)) {
          setAddDialogOpen(!addDialogOpen)
        }

        // Button 9 (Start/Menu): Toggle Settings view
        if (justPressed(9)) {
          setView(view === "settings" ? "library" : "settings")
        }
      }

      animId = requestAnimationFrame(poll)
    }

    animId = requestAnimationFrame(poll)
    return () => cancelAnimationFrame(animId)
  }, [
    games,
    view,
    openGameId,
    addDialogOpen,
    focusedIndex,
    gamepadActive,
    handleBumper,
    setOpenGameId,
    setAddDialogOpen,
    setView,
    togglePlay,
  ])

  return {
    connected,
    gamepadName,
    gamepadActive,
    focusedIndex,
    setFocusedIndex,
  }
}
