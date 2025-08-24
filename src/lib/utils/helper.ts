import { bytesToBase64Url } from '@ldclabs/cose-ts/utils'
import { encode } from 'cborg'

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

export function formatSize(bytes: number | BigInt): string {
  const n = Number(bytes)
  if (n < 1024) {
    return `${n}B`
  } else if (n < 1024 * 1024) {
    return `${(n / 1024).toFixed(2)}KB`
  } else if (n < 1024 * 1024 * 1024) {
    return `${(n / 1024 / 1024).toFixed(2)}MB`
  }
  return `${(n / 1024 / 1024 / 1024).toFixed(2)}GB`
}

export function shortId(id: string, long: boolean = false): string {
  if (long) {
    return id.length > 28 ? id.slice(0, 14) + '...' + id.slice(-14) : id
  }
  return id.length > 14 ? id.slice(0, 7) + '...' + id.slice(-7) : id
}

export function ID2Cursor(_id: number): string {
  return bytesToBase64Url(encode(_id))
}

// 高性能文件转 base64url
export async function fileToBase64Url(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onloadend = () => {
      if (reader.result) {
        const base64 = (reader.result as string)
          .split(',')[1]
          .replaceAll('+', '-')
          .replaceAll('/', '_')
          .replaceAll('=', '')
        resolve(base64)
      } else {
        reject(reader.error)
      }
    }
    reader.readAsDataURL(blob)
  })
}
