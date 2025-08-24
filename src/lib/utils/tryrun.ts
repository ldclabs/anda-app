export interface TryRunResult<T> {
  controller: AbortController
  abort: () => void
  finally(onfinally?: (res: T | null) => any): Promise<any>
}

export function tryRun<T>(
  fn: (signal: AbortSignal, abortingQue: (() => void)[]) => T | Promise<T>,
  onerror?: (err: any) => void
): TryRunResult<T> {
  const controller = new AbortController()
  const abortingQue: (() => void)[] = []
  const rt = (async () => {
    try {
      return await fn(controller.signal, abortingQue)
    } catch (err: any) {
      if (controller.signal.aborted) return null
      if (onerror) {
        onerror(err)
      } else {
        console.error(err)
      }
      return null
    }
  })()

  return {
    controller,
    abort: (reason = 'tryRun aborted') => {
      controller.abort(reason)
      abortingQue.forEach((aborting) => aborting())
    },
    finally: (onfinally) => rt.then((res) => (onfinally ? onfinally(res) : res))
  }
}

export function errMessage(err: any): string {
  if (err?.data) {
    return JSON.stringify(err.data, (key, value) =>
      typeof value === 'bigint' ? value.toString() : value
    )
  }
  if (err?.message) {
    return err.message
  }
  return String(err)
}
