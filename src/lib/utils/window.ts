import debounce from 'debounce'

const STR_UNDEFINED = 'undefined'

// NOTE: Use the function to guarantee it's re-evaluated between jsdom and node runtime for tests.
export const isWindowDefined = typeof window != STR_UNDEFINED
export const isDocumentDefined = typeof document != STR_UNDEFINED
export const isNotificationSupported =
  isWindowDefined && 'Notification' in window
export const hasRequestAnimationFrame = () =>
  isWindowDefined && typeof window['requestAnimationFrame'] != STR_UNDEFINED
export const noop = () => {}

let online = true
export const isOnline = () => online

// For node and React Native, `add/removeEventListener` doesn't exist on window.
const [onWindowEvent, offWindowEvent] =
  isWindowDefined && window.addEventListener
    ? [
        window.addEventListener.bind(window),
        window.removeEventListener.bind(window)
      ]
    : [noop, noop]

export const isVisible = () => {
  const visibilityState = isDocumentDefined && document.visibilityState
  return visibilityState == null || visibilityState !== 'hidden'
}

export const isActive = () => isOnline() && isVisible()

export const initFocus = (callback: (ev: Event) => void) => {
  // TODO: callback will be triggered 2 times when the page is focused.
  if (isDocumentDefined) {
    document.addEventListener('visibilitychange', callback)
  } else {
    onWindowEvent('focus', callback)
  }
  return () => {
    if (isDocumentDefined) {
      document.removeEventListener('visibilitychange', callback)
    }
    offWindowEvent('focus', callback)
  }
}

export const initReconnect = (
  onlineCallback: () => void = noop,
  offlineCallback: () => void = noop
) => {
  const onOnline = () => {
    online = true
    onlineCallback()
  }
  const onOffline = () => {
    online = false
    offlineCallback()
  }
  onWindowEvent('online', onOnline)
  onWindowEvent('offline', onOffline)
  return () => {
    offWindowEvent('online', onOnline)
    offWindowEvent('offline', onOffline)
  }
}

export function clickOutside(node: HTMLElement, callback: () => void = noop) {
  const handler = (ev: PointerEvent) => {
    if (!node.contains(ev.target as Node)) {
      callback()
    }
  }

  onWindowEvent('pointerup', handler)

  return () => {
    offWindowEvent('pointerup', handler)
  }
}

export function scrollOnHooks(
  node: HTMLElement,
  {
    onTop,
    onBottom,
    onMoveUp,
    onMoveDown,
    inMoveUpViewport,
    inMoveDownViewport,
    inViewportHasId = true,
    inViewportHasClass = ''
  }: {
    onTop?: (() => void) | undefined
    onBottom?: (() => void) | undefined
    onMoveUp?: (() => void) | undefined
    onMoveDown?: (() => void) | undefined
    inMoveUpViewport?: ((els: HTMLElement[]) => void) | undefined
    inMoveDownViewport?: ((els: HTMLElement[]) => void) | undefined
    inViewportHasId?: boolean
    inViewportHasClass?: string
  }
) {
  if (!node) return noop

  // 减少 debounce 延迟，提升响应性
  const callTop = onTop && debounce(onTop, 200, { immediate: false })
  const callBottom = onBottom && debounce(onBottom, 200, { immediate: false })
  const callMoveUp = onMoveUp && debounce(onMoveUp, 100, { immediate: false })
  const callMoveDown =
    onMoveDown && debounce(onMoveDown, 100, { immediate: false })
  const callInMoveUpViewport =
    inMoveUpViewport && debounce(inMoveUpViewport, 100, { immediate: false })
  const callInMoveDownViewport =
    inMoveDownViewport &&
    debounce(inMoveDownViewport, 100, { immediate: false })

  // 初始化 lastScrollTop 为当前位置
  let lastScrollTop = node.scrollTop
  let isInitialized = false

  // 添加状态跟踪，避免重复触发
  let hasTriggeredTop = false
  let hasTriggeredBottom = false

  const handler = (ev: Event) => {
    const target = ev.currentTarget as HTMLElement
    const currentScrollTop = target.scrollTop
    const scrollDirection = currentScrollTop > lastScrollTop ? 'down' : 'up'

    // 第一次滚动时初始化状态
    if (!isInitialized) {
      isInitialized = true
      hasTriggeredTop = currentScrollTop <= 10
      hasTriggeredBottom =
        target.clientHeight + currentScrollTop + 10 >= target.scrollHeight
    }

    // 处理滚动方向回调
    if (scrollDirection === 'down') {
      callMoveUp && callMoveUp()
      if (callInMoveUpViewport) {
        let children = Array.from(target.children) as HTMLElement[]
        if (inViewportHasId || inViewportHasClass) {
          children = children.filter((el) => {
            if (inViewportHasId) {
              return !!el.id
            }
            return el.classList.contains(inViewportHasClass)
          })
        }
        const els = elementsInViewport(target, children)
        if (els.length > 0) {
          callInMoveUpViewport(els)
        }
      }
    } else if (scrollDirection === 'up') {
      callMoveDown && callMoveDown()
      if (callInMoveDownViewport) {
        let children = Array.from(target.children) as HTMLElement[]
        if (inViewportHasId || inViewportHasClass) {
          children = children.filter((el) => {
            if (inViewportHasId) {
              return !!el.id
            }
            return el.classList.contains(inViewportHasClass)
          })
        }
        const els = elementsInViewport(target, children)
        if (els.length > 0) {
          callInMoveDownViewport(els)
        }
      }
    }

    // 顶部检测：接近顶部且之前未触发过
    if (currentScrollTop <= 10 && !hasTriggeredTop) {
      hasTriggeredTop = true
      hasTriggeredBottom = false
      callTop && callTop()
    }

    // 底部检测：接近底部且之前未触发过
    const isNearBottom =
      target.clientHeight + currentScrollTop + 10 >= target.scrollHeight
    if (isNearBottom && !hasTriggeredBottom) {
      hasTriggeredBottom = true
      hasTriggeredTop = false
      callBottom && callBottom()
    }

    // 重置状态：如果离开边界区域
    if (currentScrollTop > 50) {
      hasTriggeredTop = false
    }
    if (target.clientHeight + currentScrollTop + 50 < target.scrollHeight) {
      hasTriggeredBottom = false
    }

    lastScrollTop = currentScrollTop
  }

  node.addEventListener('scroll', handler)
  return () => {
    node.removeEventListener('scroll', handler)
    callTop && callTop.clear()
    callBottom && callBottom.clear()
    callMoveUp && callMoveUp.clear()
    callMoveDown && callMoveDown.clear()
    callInMoveUpViewport && callInMoveUpViewport.clear()
    callInMoveDownViewport && callInMoveDownViewport.clear()
  }
}

export function elementsInViewport(
  container: HTMLElement,
  els: HTMLElement[]
): HTMLElement[] {
  const containerRect = container.getBoundingClientRect()
  const rt: HTMLElement[] = []
  for (const el of els) {
    const rect = el.getBoundingClientRect()
    const threadhold = 0.5 * rect.height
    if (
      (rect.top >= containerRect.top &&
        rect.top + threadhold < containerRect.bottom) ||
      (rect.bottom <= containerRect.bottom &&
        rect.bottom - threadhold > containerRect.top)
    ) {
      rt.push(el)
    }
  }

  return rt
}
