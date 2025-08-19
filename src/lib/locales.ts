export const translations: Record<string, Record<string, string>> = {
  // Settings
  'settings.title': {
    en: 'Settings',
    zh: '设置'
  },
  'settings.description': {
    en: 'Configure application appearance, language and functionality',
    zh: '配置应用程序的外观、语言和功能设置'
  },

  // Navigation
  'settings.nav.general': {
    en: 'General',
    zh: '通用'
  },
  'settings.nav.appearance': {
    en: 'Appearance',
    zh: '外观'
  },
  'settings.nav.network': {
    en: 'Network',
    zh: '网络'
  },
  'settings.nav.ai': {
    en: 'AI Configuration',
    zh: 'AI 配置'
  },

  // General settings
  'settings.general.title': {
    en: 'General',
    zh: '通用'
  },
  'settings.general.description': {
    en: 'Configure basic application settings',
    zh: '配置应用程序的基本设置'
  },
  'settings.language.title': {
    en: 'Language',
    zh: '语言'
  },
  'settings.language.description': {
    en: 'Select the display language for the application',
    zh: '选择应用程序的显示语言'
  },

  // Appearance settings
  'settings.appearance.title': {
    en: 'Appearance',
    zh: '外观'
  },
  'settings.appearance.description': {
    en: 'Customize the appearance and theme of the application',
    zh: '自定义应用程序的外观和主题'
  },
  'settings.theme.title': {
    en: 'Theme',
    zh: '主题'
  },
  'settings.theme.description': {
    en: 'Choose light, dark, or follow system settings',
    zh: '选择浅色、深色或跟随系统设置'
  },
  'settings.theme.light': {
    en: 'Light',
    zh: '浅色'
  },
  'settings.theme.dark': {
    en: 'Dark',
    zh: '深色'
  },
  'settings.theme.system': {
    en: 'System',
    zh: '系统'
  },

  // Network settings
  'settings.network.title': {
    en: 'Network',
    zh: '网络'
  },
  'settings.network.description': {
    en: 'Configure network connections and proxy settings',
    zh: '配置网络连接和代理设置'
  },
  'settings.proxy.title': {
    en: 'HTTPS Proxy',
    zh: 'HTTPS 代理'
  },
  'settings.proxy.description': {
    en: 'Set network proxy server address, leave empty to disable proxy',
    zh: '设置网络代理服务器地址，留空表示不使用代理'
  },
  'settings.proxy.placeholder': {
    en: 'e.g.: http://127.0.0.1:7890',
    zh: '例如: http://127.0.0.1:7890'
  },

  // AI settings - 添加缺少的键
  'settings.ai.provider.title': {
    en: 'AI Provider',
    zh: 'AI 提供商'
  },
  'settings.ai.provider.description': {
    en: 'Choose your preferred AI service provider',
    zh: '选择你的首选 AI 服务提供商'
  },
  'settings.ai.model': {
    en: 'Model',
    zh: '模型'
  },
  'settings.ai.api_key': {
    en: 'API Key',
    zh: 'API 密钥'
  },
  'settings.ai.api_base': {
    en: 'API Base URL',
    zh: 'API Base URL'
  },
  'settings.ai.save': {
    en: 'Save AI Configuration',
    zh: '保存 AI 配置'
  },

  // Messages
  'settings.optional': {
    en: 'Optional',
    zh: '可选'
  },
  'settings.saved': {
    en: 'Settings saved',
    zh: '设置已保存'
  },
  'settings.save_failed': {
    en: 'Failed to save settings',
    zh: '设置保存失败'
  },
  'settings.load_failed': {
    en: 'Failed to load settings',
    zh: '加载设置失败'
  },
  'settings.saving': {
    en: 'Saving...',
    zh: '保存中...'
  },

  // Language options
  'language.english': {
    en: 'English',
    zh: 'English'
  },
  'language.chinese': {
    en: '中文',
    zh: '中文'
  },

  // App
  'app.restart_update': {
    en: 'Restart to update',
    zh: '重启并更新'
  },
  'app.download_update': {
    en: 'Downloading %{version}',
    zh: '下载 %{version}'
  },
  'app.sign_in': {
    en: 'Sign In',
    zh: '登录'
  },
  'app.sign_in_fallback.title': {
    en: 'Sign In by authentication URL',
    zh: '通过身份验证 URL 登录'
  },
  'app.sign_in_fallback.by_url': {
    en: 'Sign In by URL',
    zh: '通过 URL 登录'
  },
  'app.sign_in_fallback.again': {
    en: 'Sign In Again',
    zh: '重新登录'
  },
  'app.log_out': {
    en: 'Logout',
    zh: '退出登录'
  },
  'app.copy_text': {
    en: 'Copy Text',
    zh: '复制纯文本'
  },
  'app.copy_origin': {
    en: 'Copy origin',
    zh: '复制原始内容'
  },

  // Assistant
  'assistant.title': {
    en: 'Assistant',
    zh: 'AI 助手'
  },
  'assistant.thinking': {
    en: 'Thinking...',
    zh: '思考中...'
  },
  'assistant.retry': {
    en: 'Retry',
    zh: '重试'
  },
  'assistant.run': {
    en: 'Run',
    zh: '运行'
  },
  'assistant.not_ready': {
    en: 'Please configure AI service provider:',
    zh: '请先配置 AI 服务提供商：'
  },
  'assistant.signin_required': {
    en: 'Please sign in to start chatting:',
    zh: '请先登录以开始对话：'
  },
  'assistant.prompt.placeholder': {
    en: 'Type your message...',
    zh: '输入你的消息...'
  },
  'assistant.hello.title_with_name': {
    en: 'Hello, I am %{name}',
    zh: '你好，我是 %{name}'
  },
  'assistant.hello.title': {
    en: 'Give me a name and let our journey begin',
    zh: '为我命名，开启进化之旅'
  },
  'assistant.hello.description': {
    en: '---\n\nI am an AI companion that evolves with you.\n\n● **Shared Growth**\nForged by [KIP](https://github.com/ldclabs/KIP), my unique identity will be shaped by our conversations.\n\n● **Web3 Identity**\nWith [ICP](https://internetcomputer.org/) authentication, we can access the new Agentic Network.',
    zh: '---\n\n我是一个为你而生的进化型 AI 伙伴。\n\n● **共同成长**\n基于 [KIP](https://github.com/ldclabs/KIP) 构建，我们的会话将塑造独一无二的我。\n\n● **Web3 身份**\n使用 [ICP](https://internetcomputer.org/) 身份验证，进入全新的智能体网络。'
  }
}
