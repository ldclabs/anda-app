export async function sleep(ms: number): Promise<void> {
  return new Promise((res) => setTimeout(res, ms))
}

export function formatDateTime(ts: bigint | number): string {
  const now = Date.now()
  const t = Number(ts)
  if (t >= now - 24 * 3600 * 1000) {
    return new Date(t).toLocaleTimeString()
  } else if (t >= now - 7 * 24 * 3600 * 1000) {
    return new Date(t).toLocaleTimeString(undefined, { weekday: 'long' })
  }

  return new Date(t).toLocaleDateString()
}
