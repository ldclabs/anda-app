import type { ChatMessage, ChatSession } from '$lib/types/chat'

class ChatStore {
  private _sessions = $state<ChatSession[]>([])
  private _currentSessionId = $state<string | null>(null)
  private _isLoading = $state(false)

  get sessions() {
    return this._sessions
  }

  get currentSession() {
    return this._sessions.find(s => s.id === this._currentSessionId) || null
  }

  get isLoading() {
    return this._isLoading
  }

  createSession(title: string = 'New Chat'): string {
    const sessionId = crypto.randomUUID()
    const newSession: ChatSession = {
      id: sessionId,
      title,
      messages: [],
      createdAt: new Date(),
      updatedAt: new Date()
    }
    
    this._sessions.unshift(newSession)
    this._currentSessionId = sessionId
    return sessionId
  }

  selectSession(sessionId: string) {
    this._currentSessionId = sessionId
  }

  addMessage(content: string, role: 'user' | 'assistant') {
    if (!this.currentSession) {
      this.createSession()
    }

    const message: ChatMessage = {
      id: crypto.randomUUID(),
      content,
      role,
      timestamp: new Date()
    }

    this.currentSession!.messages.push(message)
    this.currentSession!.updatedAt = new Date()

    // Update session title based on first user message
    if (role === 'user' && this.currentSession!.messages.length === 1) {
      this.currentSession!.title = content.slice(0, 30) + (content.length > 30 ? '...' : '')
    }
  }

  async sendMessage(content: string) {
    this._isLoading = true
    
    // Add user message
    this.addMessage(content, 'user')
    
    try {
      // Add typing indicator
      const typingMessage: ChatMessage = {
        id: 'typing',
        content: '',
        role: 'assistant',
        timestamp: new Date(),
        isTyping: true
      }
      this.currentSession!.messages.push(typingMessage)
      
      // Simulate API call (replace with actual API call)
      await new Promise(resolve => setTimeout(resolve, 1000 + Math.random() * 2000))
      
      // Remove typing indicator
      const typingIndex = this.currentSession!.messages.findIndex(m => m.id === 'typing')
      if (typingIndex !== -1) {
        this.currentSession!.messages.splice(typingIndex, 1)
      }
      
      // Add AI response
      const responses = [
        "我理解你的问题。让我为你提供一个详细的解答...",
        "这是一个很好的问题！基于我的分析，我建议...",
        "根据你提供的信息，我可以帮你解决这个问题。首先...",
        "让我来帮你分析一下这个情况。从几个方面来看...",
        "我明白你的需求。这里有几个解决方案供你参考..."
      ]
      
      const response = responses[Math.floor(Math.random() * responses.length)]
      this.addMessage(response, 'assistant')
      
    } catch (error) {
      console.error('Failed to send message:', error)
      this.addMessage('抱歉，发送消息时出现了错误，请稍后重试。', 'assistant')
    } finally {
      this._isLoading = false
    }
  }

  deleteSession(sessionId: string) {
    const index = this._sessions.findIndex(s => s.id === sessionId)
    if (index !== -1) {
      this._sessions.splice(index, 1)
      if (this._currentSessionId === sessionId) {
        this._currentSessionId = this._sessions.length > 0 ? this._sessions[0].id : null
      }
    }
  }

  clearCurrentSession() {
    if (this.currentSession) {
      this.currentSession.messages = []
      this.currentSession.updatedAt = new Date()
    }
  }
}

export const chatStore = new ChatStore()