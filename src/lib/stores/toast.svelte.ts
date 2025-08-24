import { errMessage, tryRun, type TryRunResult } from '$lib/utils/tryrun'

export interface ToastModel {
  id: number
  type: 'success' | 'error' | 'info'
  message?: string
  icon?: () => any
  content?: () => any
  duration?: number // ms
  dismissable?: boolean
  onclick: () => void
}

export const toastStore = $state<ToastModel[]>([])

let idCounter = 0
export function triggerToast(toast: Omit<ToastModel, 'id' | 'onclick'>) {
  const id = ++idCounter
  const model: ToastModel = {
    id,
    type: toast.type,
    message: toast.message,
    icon: toast.icon,
    content: toast.content,
    duration:
      toast.duration ??
      (toast.type === 'success'
        ? 3000
        : toast.type === 'error'
          ? 30000
          : 10000),
    dismissable: toast.dismissable ?? true,
    onclick: () => removeToast(id)
  }
  toastStore.push(model)

  // auto remove
  setTimeout(() => {
    removeToast(id)
  }, model.duration)
}

export const ErrorLogs = $state<Error[]>([])

export function toastRun<T>(
  fn: (signal: AbortSignal, abortingQue: (() => void)[]) => T | Promise<T>,
  errMsg?: string
): TryRunResult<T> {
  return tryRun(fn, (err: any) => {
    if (err) {
      console.error(err)
      ErrorLogs.push(err)
      if (ErrorLogs.length > 200) {
        ErrorLogs.splice(0, 10)
      }
      triggerToast({
        type: 'error',
        message: errMsg ?? errMessage(err)
      })
    }
  })
}

function removeToast(id: number) {
  const idx = toastStore.findIndex((t) => t.id === id)
  if (idx >= 0) {
    toastStore.splice(idx, 1)
  }
}
