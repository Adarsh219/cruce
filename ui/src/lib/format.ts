export function formatPlaytime(min: number) {
  if (min <= 0) return "0m"
  const h = Math.floor(min / 60)
  const m = min % 60
  if (h === 0) return `${m}m`
  return `${h.toLocaleString("en-US")}h ${m}m`
}

export function formatLastPlayed(days: number | null) {
  if (days === null) return "Never"
  if (days === 0) return "Today"
  if (days === 1) return "Yesterday"
  if (days < 7) return `${days} days ago`
  const weeks = Math.floor(days / 7)
  return weeks === 1 ? "Last week" : `${weeks} weeks ago`
}

export function initials(title: string) {
  return title
    .replace(/[^a-zA-Z0-9 ]/g, "")
    .split(" ")
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase())
    .join("")
}

export function fuzzyScore(query: string, text: string) {
  const q = query.toLowerCase().replace(/\s+/g, "")
  const t = text.toLowerCase()
  if (!q) return 1
  if (t.includes(q)) return 100 - t.indexOf(q)
  let ti = 0
  let score = 0
  let streak = 0
  for (const ch of q) {
    const found = t.indexOf(ch, ti)
    if (found === -1) return 0
    streak = found === ti ? streak + 1 : 0
    score += 1 + streak
    ti = found + 1
  }
  return score
}
