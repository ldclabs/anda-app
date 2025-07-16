import katex from 'katex'
import MarkdownIt from 'markdown-it'
import mermaid from 'mermaid'
import Prism from 'prismjs'
import components from 'prismjs/components'

// 导入 Prism 语言支持
import './prismjs'

// 初始化 Mermaid
mermaid.initialize({
  startOnLoad: false,
  theme: 'default',
  securityLevel: 'loose',
  fontFamily: 'monospace'
})

// 创建 MarkdownIt 实例
const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
  breaks: true
})

md.linkify.set({ fuzzyLink: false, fuzzyEmail: false })
// 自定义链接渲染规则，让链接在新页面打开
const defaultLinkOpenRenderer =
  md.renderer.rules.link_open ||
  function (tokens, idx, options, env, renderer) {
    return renderer.renderToken(tokens, idx, options)
  }

md.renderer.rules.link_open = function (tokens, idx, options, env, renderer) {
  const token = tokens[idx]

  // 添加 target="_blank" 和 rel="noopener noreferrer" 属性
  token.attrSet('target', '_blank')
  token.attrSet('rel', 'noopener noreferrer')

  return defaultLinkOpenRenderer(tokens, idx, options, env, renderer)
}

// KaTeX 插件 - 处理数学公式
function katexPlugin(md: MarkdownIt) {
  // 行内数学公式 $...$
  md.inline.ruler.before('escape', 'math_inline', function (state, silent) {
    const start = state.pos
    if (state.src[start] !== '$') return false

    let pos = start + 1
    while (pos < state.posMax && state.src[pos] !== '$') {
      if (state.src[pos] === '\\') pos++ // 跳过转义字符
      pos++
    }

    if (pos >= state.posMax || state.src[pos] !== '$') return false

    const content = state.src.slice(start + 1, pos)
    if (!content.trim()) return false

    if (!silent) {
      const token = state.push('math_inline', 'math', 0)
      token.content = content
      token.markup = '$'
    }

    state.pos = pos + 1
    return true
  })

  // 块级数学公式 $$...$$
  md.block.ruler.before(
    'fence',
    'math_block',
    function (state, start, end, silent) {
      const marker = '$$'
      let pos = state.bMarks[start] + state.tShift[start]
      let max = state.eMarks[start]

      if (pos + marker.length > max) return false
      if (state.src.slice(pos, pos + marker.length) !== marker) return false

      pos += marker.length
      let firstLine = state.src.slice(pos, max).trim()

      if (firstLine.endsWith(marker)) {
        firstLine = firstLine.slice(0, -marker.length).trim()
        let nextLine = start
        let content = firstLine

        if (!silent) {
          const token = state.push('math_block', 'math', 0)
          token.content = content
          token.markup = marker
          token.map = [start, nextLine + 1]
        }

        state.line = nextLine + 1
        return true
      }

      let nextLine = start + 1
      let content = firstLine

      while (nextLine < end) {
        pos = state.bMarks[nextLine] + state.tShift[nextLine]
        max = state.eMarks[nextLine]

        if (pos < max && state.tShift[nextLine] < state.blkIndent) break

        const line = state.src.slice(pos, max).trim()
        if (line === marker) {
          if (!silent) {
            const token = state.push('math_block', 'math', 0)
            token.content = content
            token.markup = marker
            token.map = [start, nextLine + 1]
          }

          state.line = nextLine + 1
          return true
        }

        content += '\n' + line
        nextLine++
      }

      return false
    }
  )

  // 渲染器
  md.renderer.rules.math_inline = function (tokens, idx) {
    const token = tokens[idx]
    try {
      return katex.renderToString(token.content, { displayMode: false })
    } catch (err) {
      return `<span class="katex-error">${token.content}</span>`
    }
  }

  md.renderer.rules.math_block = function (tokens, idx) {
    const token = tokens[idx]
    try {
      return `<div class="katex-display">${katex.renderToString(token.content, { displayMode: true })}</div>`
    } catch (err) {
      return `<div class="katex-error">${token.content}</div>`
    }
  }
}

// Mermaid 插件
function mermaidPlugin(md: MarkdownIt) {
  const fence = md.renderer.rules.fence!

  md.renderer.rules.fence = function (tokens, idx, options, env, renderer) {
    const token = tokens[idx]
    const info = token.info ? token.info.trim() : ''

    if (info === 'mermaid') {
      const id = `mermaid-${Math.random().toString(36).substr(2, 9)}`
      return `<div class="mermaid" id="${id}">${token.content}</div>`
    }

    return fence(tokens, idx, options, env, renderer)
  }
}

// 代码高亮插件
function prismPlugin(md: MarkdownIt) {
  const fence = md.renderer.rules.fence!
  const langs: Record<string, string> = {}
  for (const lang in components.languages) {
    if (Prism.languages[lang]) {
      langs[lang] = lang

      if (components.languages[lang].alias) {
        if (Array.isArray(components.languages[lang].alias)) {
          components.languages[lang].alias.forEach((alias: string) => {
            langs[alias] = lang
          })
        } else if (typeof components.languages[lang].alias === 'string') {
          langs[components.languages[lang].alias] = lang
        }
      }
    }
  }

  md.renderer.rules.fence = function (tokens, idx, options, env, renderer) {
    const token = tokens[idx]
    const info = token.info ? token.info.trim() : ''
    let langName = info.split(/\s+/g)[0]
    if (langName === 'mermaid') {
      // 让 mermaid 插件处理
      return fence(tokens, idx, options, env, renderer)
    }

    langName = langs[langName] || langName
    if (langName && Prism.languages[langName]) {
      try {
        const highlighted = Prism.highlight(
          token.content,
          Prism.languages[langName],
          langName
        )
        return `<pre class="language-${langName}"><code class="language-${langName}">${highlighted}</code></pre>`
      } catch (err) {
        console.warn('Prism highlighting failed:', err)
      }
    }

    // 回退到默认渲染
    return `<pre><code>${md.utils.escapeHtml(token.content)}</code></pre>`
  }
}

// 应用插件
md.use(katexPlugin)
md.use(mermaidPlugin)
md.use(prismPlugin)

/**
 * 渲染 Markdown 文本为 HTML
 * @param markdown - Markdown 文本
 * @param options - 渲染选项
 * @returns 渲染后的 HTML 字符串
 */
export function renderMarkdown(
  markdown: string,
  options?: {
    enableMermaid?: boolean
    enableKatex?: boolean
    enablePrism?: boolean
  }
): [string, () => Promise<void>] {
  const {
    enableMermaid = true,
    enableKatex = true,
    enablePrism = true
  } = options || {}

  try {
    let html = md.render(markdown)

    // 如果启用了 Mermaid，需要在 DOM 更新后渲染图表
    if (enableMermaid && html.includes('class="mermaid"')) {
      // 这里返回的 HTML 包含 mermaid div，需要在组件中调用 renderMermaidCharts
      return [html, renderMermaidCharts]
    }

    return [html, () => Promise.resolve()]
  } catch (e) {
    return [markdown, () => Promise.resolve()]
  }
}

/**
 * 渲染页面中的 Mermaid 图表
 * 需要在 DOM 更新后调用
 */
export async function renderMermaidCharts(): Promise<void> {
  const mermaidElements = document.querySelectorAll('.mermaid')

  for (const element of mermaidElements) {
    if (element.getAttribute('data-processed') === 'true') continue

    try {
      const id =
        element.id || `mermaid-${Math.random().toString(36).substr(2, 9)}`
      element.id = id

      const { svg } = await mermaid.render(
        id + '-svg',
        element.textContent || ''
      )
      element.innerHTML = svg
      element.setAttribute('data-processed', 'true')
    } catch (err) {
      console.error('Mermaid rendering failed:', err)
      element.innerHTML = `<pre class="mermaid-error">${element.textContent}</pre>`
    }
  }
}

/**
 * 获取 Markdown 文本的纯文本内容（去除格式）
 * @param markdown - Markdown 文本
 * @returns 纯文本内容
 */
export function getPlainText(markdown: string): string {
  const html = md.render(markdown)
  const div = document.createElement('div')
  div.innerHTML = html
  return div.textContent || div.innerText || ''
}

/**
 * 获取 Markdown 文本的摘要
 * @param markdown - Markdown 文本
 * @param maxLength - 最大长度，默认 200
 * @returns 摘要文本
 */
export function getMarkdownSummary(
  markdown: string,
  maxLength: number = 200
): string {
  const plainText = getPlainText(markdown)
  if (plainText.length <= maxLength) return plainText

  return plainText.substring(0, maxLength).trim() + '...'
}

export default {
  renderMarkdown,
  renderMermaidCharts,
  getPlainText,
  getMarkdownSummary
}
