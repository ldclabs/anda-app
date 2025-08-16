import type {
  AgentInput,
  AgentOutput,
  Conversation,
  EngineCard,
  KIPLogs,
  MemoryToolArgs,
  Resource,
  Response,
  ToolInput,
  ToolOutput
} from '$lib/types/assistant'
import { sleep } from '$lib/utils/helper'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import * as mock from './assistant.mock'

const USE_MOCK_DATA = false
const ASSISTANT_EVENT = 'AssistantReady'

export async function assistant_info(): Promise<EngineCard> {
  if (USE_MOCK_DATA) {
    return await mock.assistant_info()
  }
  return await invoke('assistant_info')
}

export async function assistant_name(): Promise<string | null> {
  return await invoke('assistant_name')
}

async function tool_call<I, O>(input: ToolInput<I>): Promise<ToolOutput<O>> {
  if (USE_MOCK_DATA) {
    return await mock.tool_call(input)
  }
  return await invoke('tool_call', { input })
}

async function agent_run<I, O>(input: AgentInput): Promise<AgentOutput> {
  if (USE_MOCK_DATA) {
    return await mock.agent_run(input)
  }
  return await invoke('agent_run', { input })
}

class AssistantStore {
  static async init() {
    const info = await assistant_info()
    assistantStore._isReady = info.agents.length > 0
    listen<boolean>(ASSISTANT_EVENT, (event) => {
      assistantStore._isReady = event.payload
    })
  }

  private _conversations = $state<Conversation[]>([])
  private _prevConversationCursor = $state<string | undefined>()
  private _latestConversationId = $state<number>(0)
  private _isLoading = $state(false)
  private _isLoadingPrev = $state(false)
  private _isChating = $state(false)
  private _user = $state<string>('2vxsx-fae')
  private _isReady = $state(false)

  get isReady() {
    return this._isReady
  }

  get conversations() {
    return this._conversations
  }

  get isLoading() {
    return this._isLoading
  }

  get isLoadingPrev() {
    return this._isLoadingPrev
  }

  get isChating() {
    return this._isChating
  }

  get latestConversationId() {
    return this._latestConversationId
  }

  reset_if_user_changed(user: string) {
    if (user == this._user) return
    this._user = user
    this._conversations = []
    this._prevConversationCursor = undefined
    this._latestConversationId = 0
    this._isLoading = false
    this._isLoadingPrev = false
    this._isChating = false
  }

  private addConversation(conversation: Conversation) {
    let idx = this._conversations.findIndex(
      (item) => item._id === conversation._id
    )
    if (idx < 0) {
      this._conversations = [...this._conversations, conversation]
      this._latestConversationId = conversation._id
    } else {
      this._conversations = [
        ...this._conversations.slice(0, idx),
        conversation,
        ...this._conversations.slice(idx + 1)
      ]
    }
  }

  private async fetchConversation(
    _id: number,
    interval: number = 2000,
    max_interval: number = 60000
  ) {
    const res: ToolOutput<Response<Conversation>> = await tool_call({
      name: 'memory_api',
      args: {
        _type: 'GetConversation',
        _id
      } as MemoryToolArgs
    })

    console.log('GetConversation', res)
    if (res.output.error) {
      console.error('GetConversation', res.output.error)
      throw res.output.error
    }

    const now_ms = Date.now()
    const conversation = res.output.result!
    this.addConversation(conversation)
    if (
      conversation.status != 'submitted' ||
      now_ms - conversation.updated_at > 600000
    ) {
      this._isChating = false
    }

    if (
      (conversation.status == 'submitted' ||
        conversation.status == 'working') &&
      now_ms - conversation.updated_at < 3600000
    ) {
      sleep(Math.min(interval, max_interval)).then(() => {
        this.fetchConversation(_id, Math.floor(interval * 1.2), max_interval)
      })
    }
  }

  async listKipLogs(cursor?: string, limit?: number) {
    const res: ToolOutput<Response<KIPLogs[]>> = await tool_call({
      name: 'memory_api',
      args: {
        _type: 'ListKipLogs',
        cursor,
        limit
      } as MemoryToolArgs
    })

    console.log('ListKipLogs', res)
    if (res.output.error) {
      console.error('ListKipLogs', res.output.error)
      throw res.output.error
    }

    return res.output
  }

  async loadPreviousConversations(): Promise<boolean> {
    if (!this._prevConversationCursor || this._isLoadingPrev) return false

    this._isLoadingPrev = true
    try {
      const [_, res]: [void, ToolOutput<Response<Conversation[]>>] =
        await Promise.all([
          sleep(300),
          tool_call<MemoryToolArgs, Response<Conversation[]>>({
            name: 'memory_api',
            args: {
              _type: 'ListPrevConversations',
              cursor: this._prevConversationCursor,
              limit: 20
            } as MemoryToolArgs
          })
        ])

      console.log('loadPreviousConversations', res)
      if (res.output.error) {
        throw res.output.error
      }

      const conversations = res.output.result!
      if (conversations.length > 0) {
        this._conversations = [...conversations, ...this._conversations]
        this._prevConversationCursor = res.output.next_cursor
        return !!this._prevConversationCursor
      }
    } catch (error) {
      console.error('ListPrevConversations', error)
      throw error
    } finally {
      this._isLoadingPrev = false
    }

    return false
  }

  async loadLatestConversations() {
    if (this._isLoading) return
    this._isLoading = true
    try {
      const res: ToolOutput<Response<Conversation[]>> = await tool_call({
        name: 'memory_api',
        args: {
          _type: 'ListPrevConversations',
          limit: 20
        } as MemoryToolArgs
      })

      console.log('loadLatestConversations', res)
      if (res.output.error) {
        throw res.output.error
      }

      const conversations = res.output.result!
      if (!conversations.length) {
        this._conversations = []
        this._prevConversationCursor = undefined
        this._latestConversationId = 0
        return
      }

      this._latestConversationId = conversations.at(-1)!._id
      let prev = conversations.at(0)!
      let idx = this._conversations.findIndex((item) => item._id === prev._id)
      if (idx < 0) {
        this._conversations = conversations
        this._prevConversationCursor = res.output.next_cursor
      } else {
        this._conversations = [
          ...this._conversations.slice(0, idx),
          ...conversations
        ]
      }
    } catch (error) {
      console.error('loadLatestConversations', error)
      throw error
    } finally {
      this._isLoading = false
    }
  }

  async chat(content: string, resources?: Resource[]) {
    const prompt = content.trim()
    if (!prompt) return

    this._isChating = true
    try {
      const res = await agent_run({
        name: 'assistant',
        prompt,
        resources
      })

      if (res.conversation) {
        await this.fetchConversation(res.conversation)
      }

      if (res.failed_reason) {
        throw res.failed_reason
      }
    } catch (error) {
      this._isChating = false
      console.error('AssistantStore.chat', error, content, resources)
      throw error
    }
  }
}

export const assistantStore = new AssistantStore()

AssistantStore.init().catch((err) => {
  console.error('Failed to initialize assistant store', err)
})
