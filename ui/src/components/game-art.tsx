import { initials } from "@/lib/format"
import type { Game } from "@/lib/types"
import { cn } from "@/lib/utils"

export function GameIcon({ game, className }: { game: Game; className?: string }) {
  return (
    <span
      aria-hidden="true"
      className={cn(
        "relative flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-white/15 text-[11px] font-bold text-white shadow-lg",
        className,
      )}
      style={{
        background: `linear-gradient(135deg, oklch(0.62 0.17 ${game.hue}), oklch(0.32 0.1 ${game.hue + 30}))`,
      }}
    >
      {game.cover ? (
        <img
          src={game.cover}
          alt=""
          className="absolute inset-0 size-full object-cover object-top scale-[2.2]"
          loading="lazy"
        />
      ) : (
        initials(game.title)
      )}
    </span>
  )
}

export function FallbackCover({ game, className }: { game: Game; className?: string }) {
  const h = game.hue
  return (
    <div
      className={cn("relative h-full w-full overflow-hidden", className)}
      style={{
        background: `radial-gradient(120% 80% at 0% 0%, oklch(0.45 0.14 ${h}) 0%, transparent 60%), radial-gradient(100% 70% at 100% 100%, oklch(0.32 0.12 ${h + 40}) 0%, transparent 60%), oklch(0.16 0.03 ${h})`,
      }}
    >
      <div
        className="absolute -top-1/4 -right-1/4 aspect-square w-[90%] rotate-45 rounded-[22%] border"
        style={{ borderColor: `oklch(0.75 0.15 ${h} / 0.25)` }}
      />
      <div
        className="absolute top-[18%] right-[8%] aspect-square w-[55%] rotate-45 rounded-[22%] border-2"
        style={{ borderColor: `oklch(0.8 0.15 ${h} / 0.35)` }}
      />
      <div
        className="absolute bottom-[28%] -left-[12%] aspect-square w-[45%] rounded-full"
        style={{ background: `oklch(0.7 0.16 ${h} / 0.18)` }}
      />
      <div className="absolute inset-0 bg-[repeating-linear-gradient(135deg,transparent_0_14px,rgba(255,255,255,0.025)_14px_15px)]" />
      <span
        className="absolute top-[34%] left-1/2 -translate-x-1/2 -translate-y-1/2 text-5xl font-black tracking-tighter"
        style={{ color: `oklch(0.92 0.06 ${h})` }}
      >
        {initials(game.title)}
      </span>
    </div>
  )
}

export function CoverArt({ game }: { game: Game; sizes?: string; priority?: boolean }) {
  if (!game.cover) return <FallbackCover game={game} />
  return (
    <img
      src={game.cover}
      alt={`${game.title} cover art`}
      className="absolute inset-0 size-full object-cover transition-transform duration-500 group-hover:scale-[1.04]"
      loading="lazy"
    />
  )
}
