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
  theme: 'neutral',
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
  // 定义支持的分隔符
  const delimiters = [
    { left: '$$', right: '$$', block: true, inline: true },
    { left: '$', right: '$', block: false, inline: true },
    { left: '\\(', right: '\\)', block: true, inline: true },
    { left: '\\[', right: '\\]', block: true, inline: true }
  ]

  // 行内数学公式解析器
  md.inline.ruler.before('escape', 'math_inline', function (state, silent) {
    const start = state.pos
    const src = state.src

    // 检查所有行内分隔符
    for (const delimiter of delimiters) {
      if (!delimiter.inline) continue // 跳过块级分隔符

      const leftDelim = delimiter.left
      const rightDelim = delimiter.right

      // 检查是否匹配左分隔符
      if (!src.slice(start).startsWith(leftDelim)) continue

      let pos = start + leftDelim.length
      let found = false

      // 查找右分隔符
      while (pos < state.posMax) {
        if (src.slice(pos).startsWith(rightDelim)) {
          found = true
          break
        }
        if (src[pos] === '\\') pos++ // 跳过转义字符
        pos++
      }

      if (!found) continue

      const content = src.slice(start + leftDelim.length, pos)
      if (!content.trim()) continue

      if (!silent) {
        const token = state.push('math_inline', 'math', 0)
        token.content = content
        token.markup = leftDelim
      }

      state.pos = pos + rightDelim.length
      return true
    }

    return false
  })

  // 块级数学公式解析器
  md.block.ruler.before(
    'fence',
    'math_block',
    function (state, start, end, silent) {
      let pos = state.bMarks[start] + state.tShift[start]
      let max = state.eMarks[start]
      const src = state.src

      // 检查所有块级分隔符
      for (const delimiter of delimiters) {
        if (!delimiter.block) continue // 跳过行内分隔符

        const leftDelim = delimiter.left
        const rightDelim = delimiter.right

        // 检查是否匹配左分隔符
        if (!src.slice(pos).startsWith(leftDelim)) continue

        pos += leftDelim.length
        let firstLine = src.slice(pos, max).trim()

        // 处理单行情况（如 $$formula$$）
        if (firstLine.endsWith(rightDelim)) {
          const content = firstLine.slice(0, -rightDelim.length).trim()

          if (!silent) {
            const token = state.push('math_block', 'math', 0)
            token.content = content
            token.markup = leftDelim
            token.map = [start, start + 1]
          }

          state.line = start + 1
          return true
        }

        // 处理多行情况
        let nextLine = start + 1
        let content = firstLine
        let found = false

        while (nextLine < end) {
          pos = state.bMarks[nextLine] + state.tShift[nextLine]
          max = state.eMarks[nextLine]

          if (pos < max && state.tShift[nextLine] < state.blkIndent) break

          const line = src.slice(pos, max)

          // 检查是否包含右分隔符
          const rightIndex = line.indexOf(rightDelim)
          if (rightIndex !== -1) {
            // 找到右分隔符
            const beforeRight = line.slice(0, rightIndex).trim()
            if (beforeRight) {
              content += '\n' + beforeRight
            }
            found = true
            break
          }

          content += '\n' + line.trim()
          nextLine++
        }

        if (found) {
          if (!silent) {
            const token = state.push('math_block', 'math', 0)
            token.content = content
            token.markup = leftDelim
            token.map = [start, nextLine + 1]
          }

          state.line = nextLine + 1
          return true
        }
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
      return `<div class="katex-block">${katex.renderToString(token.content.trim(), { displayMode: true })}</div>`
    } catch (err) {
      return `<div class="katex-error">${token.content.trim()}</div>`
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
    } else if (langName === 'katex') {
      try {
        return `<div class="katex-block">${katex.renderToString(token.content.trim(), { displayMode: true })}</div>`
      } catch (err) {
        return `<div class="katex-error">${token.content}</div>`
      }
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
