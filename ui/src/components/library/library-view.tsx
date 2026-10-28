import { ArrowDownUp, Plus, RefreshCw, Search, SearchX, X } from "lucide-react"
import { useEffect, useMemo, useRef, useState } from "react"
import { formatPlaytime, fuzzyScore } from "@/lib/format"
import { cn } from "@/lib/utils"
import { useCruce } from "../store"
import { Btn, Kbd, Segmented } from "../ui"
import { EmptyLibrary } from "./empty-library"
import { GameCard } from "./game-card"

type Filter = "all" | "playing" | "recent" | "unplayed"
type Sort = "recent" | "az" | "playtime"

export function LibraryView() {
  const { games, setAddDialogOpen, toast, gamepad, rescan } = useCruce()
  const [query, setQuery] = useState("")
  const [filter, setFilter] = useState<Filter>("all")
  const [sort, setSort] = useState<Sort>("recent")
  const [refreshing, setRefreshing] = useState(false)
  const searchRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault()
        searchRef.current?.focus()
        searchRef.current?.select()
      }
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [])

  const counts = useMemo(
    () => ({
      all: games.length,
      playing: games.filter((g) => g.status !== "idle").length,
      recent: games.filter((g) => g.lastPlayedDaysAgo !== null && g.lastPlayedDaysAgo <= 7).length,
      unplayed: games.filter((g) => g.lastPlayedDaysAgo === null).length,
    }),
    [games],
  )

  const visible = useMemo(() => {
    let list = games.filter((g) => {
      if (filter === "playing") return g.status !== "idle"
      if (filter === "recent") return g.lastPlayedDaysAgo !== null && g.lastPlayedDaysAgo <= 7
      if (filter === "unplayed") return g.lastPlayedDaysAgo === null
      return true
    })
    if (query.trim()) {
      return list
        .map((g) => ({ g, s: fuzzyScore(query, g.title) }))
        .filter((x) => x.s > 0)
        .sort((a, b) => b.s - a.s)
        .map((x) => x.g)
    }
    list = [...list].sort((a, b) => {
      if (sort === "az") return a.title.localeCompare(b.title)
      if (sort === "playtime") return b.playtimeMin - a.playtimeMin
      return (a.lastPlayedDaysAgo ?? 9999) - (b.lastPlayedDaysAgo ?? 9999)
    })
    return list
  }, [games, filter, query, sort])

  const totalPlaytime = games.reduce((s, g) => s + g.playtimeMin, 0)

  const refresh = async () => {
    setRefreshing(true)
    try {
      await rescan()
    } finally {
      setRefreshing(false)
    }
  }

  if (games.length === 0) return <EmptyLibrary />

  return (
    <div className="mx-auto flex w-full max-w-[1600px] flex-col gap-6 px-4 py-6 sm:px-8 sm:py-8">
      <header className="flex flex-col gap-5">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <h1 className="text-2xl font-semibold tracking-tight text-balance sm:text-3xl">Library</h1>
            <p className="mt-1 text-sm text-muted-foreground">
              {games.length} games · <span className="font-mono">{formatPlaytime(totalPlaytime)}</span> played
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Btn variant="ghost" size="icon" onClick={refresh} aria-label="Refresh library">
              <RefreshCw className={cn(refreshing && "animate-spin")} />
            </Btn>
            <Btn variant="primary" onClick={() => setAddDialogOpen(true)}>
              <Plus />
              Add Game
            </Btn>
          </div>
        </div>

        <div className="flex flex-col gap-3 lg:flex-row lg:items-center">
          <div className="relative flex-1 lg:max-w-md">
            <Search className="pointer-events-none absolute top-1/2 left-3.5 size-4 -translate-y-1/2 text-muted-foreground" />
            <label htmlFor="library-search" className="sr-only">
              Search games
            </label>
            <input
              id="library-search"
              ref={searchRef}
              type="search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Escape") {
                  setQuery("")
                  e.currentTarget.blur()
                }
              }}
              placeholder="Search your library…"
              autoComplete="off"
              className="h-11 w-full rounded-xl border border-input bg-card/70 pr-20 pl-10 text-sm text-foreground placeholder:text-muted-foreground/70 outline-none transition-colors hover:border-foreground/20 focus:border-primary/60 focus:ring-2 focus:ring-ring/40 [&::-webkit-search-cancel-button]:hidden"
            />
            <div className="absolute top-1/2 right-3 flex -translate-y-1/2 items-center gap-1">
              {query ? (
                <button
                  type="button"
                  onClick={() => setQuery("")}
                  aria-label="Clear search"
                  className="rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
                >
                  <X className="size-3.5" />
                </button>
              ) : (
                <>
                  <Kbd>Ctrl</Kbd>
                  <Kbd>K</Kbd>
                </>
              )}
            </div>
          </div>

          <div className="-mx-4 overflow-x-auto px-4 scrollbar-thin lg:mx-0 lg:px-0">
            <Segmented<Filter>
              label="Filter games"
              value={filter}
              onChange={setFilter}
              options={[
                { value: "all", label: "All", count: counts.all },
                { value: "playing", label: "Playing", count: counts.playing },
                { value: "recent", label: "Recent", count: counts.recent },
                { value: "unplayed", label: "Unplayed", count: counts.unplayed },
              ]}
            />
          </div>

          <div className="relative lg:ml-auto">
            <ArrowDownUp className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
            <label htmlFor="library-sort" className="sr-only">
              Sort games
            </label>
            <select
              id="library-sort"
              value={sort}
              onChange={(e) => setSort(e.target.value as Sort)}
              className="h-10 w-full cursor-pointer appearance-none rounded-xl border border-border bg-background/50 pr-4 pl-9 text-sm font-medium text-foreground outline-none hover:border-foreground/20 focus-visible:ring-2 focus-visible:ring-ring lg:w-auto [&>option]:bg-popover"
            >
              <option value="recent">Recently played</option>
              <option value="az">Name A–Z</option>
              <option value="playtime">Most played</option>
            </select>
          </div>
        </div>
      </header>

      {visible.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-2xl border border-dashed border-border px-6 py-20 text-center">
          <SearchX className="size-8 text-muted-foreground" />
          <h2 className="mt-4 text-base font-semibold">No games match</h2>
          <p className="mt-1 max-w-sm text-sm text-muted-foreground">
            {query ? `Nothing found for “${query}”.` : "No games in this filter yet."} Try a different search or filter.
          </p>
          <Btn
            className="mt-5"
            onClick={() => {
              setQuery("")
              setFilter("all")
            }}
          >
            Reset filters
          </Btn>
        </div>
      ) : (
        <ul className="grid grid-cols-2 gap-x-4 gap-y-8 sm:grid-cols-3 sm:gap-x-6 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
          {visible.map((g, i) => (
            <li key={g.id}>
              <GameCard
                game={g}
                priority={i < 4}
                isFocused={gamepad.gamepadActive && visible[gamepad.focusedIndex]?.id === g.id}
              />
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
