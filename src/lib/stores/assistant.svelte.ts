import type {
  AgentInput,
  AgentOutput,
  Conversation,
  EngineCard,
  KIPLog,
  MemoryToolArgs,
  Resource,
  Response,
  ToolInput,
  ToolOutput
} from '$lib/types/assistant'
import { isThinking } from '$lib/types/assistant'
import { sleep } from '$lib/utils/helper'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { toastRun } from './toast.svelte'

const ASSISTANT_EVENT = 'AssistantReady'

export async function assistant_info(): Promise<EngineCard> {
  return await invoke('assistant_info')
}

export async function assistant_name(): Promise<string | null> {
  return await invoke('assistant_name')
}

export async function caller_name(): Promise<string | null> {
  return await invoke('caller_name')
}

async function tool_call<I, O>(input: ToolInput<I>): Promise<ToolOutput<O>> {
  return await invoke('tool_call', { input })
}

async function agent_run<I, O>(input: AgentInput): Promise<AgentOutput> {
  return await invoke('agent_run', { input })
}

class AssistantStore {
  static async init() {
    listen<boolean>(ASSISTANT_EVENT, (event) => {
      assistantStore._isReady = event.payload
    })

    const checkReady = async () => {
      const info = await assistant_info()
      assistantStore._isReady = info.agents.length > 0

      if (!assistantStore._isReady) {
        setTimeout(checkReady, 1000)
      }
    }

    checkReady()
  }

  private _conversations = $state<Conversation[]>([])
  private _prevConversationCursor = $state<string | undefined>()
  private _latestConversationId = $state<number>(0)
  private _isLoading = $state(false)
  private _isLoadingPrev = $state(false)
  private _isThinking = $state(0)
  private _isSubmitting = $state(false)
  private _userID = '2vxsx-fae'
  private _userName = ''
  private _isReady = $state(false)
  private _callerName = $state('')

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

  get isThinking() {
    return this._isThinking
  }

  get isSubmitting() {
    return this._isSubmitting
  }

  get latestConversationId() {
    return this._latestConversationId
  }

  get prevConversationCursor() {
    return this._prevConversationCursor
  }

  set userName(name: string) {
    this._userName = name
    caller_name().then((callerName) => {
      this._callerName = callerName || ''
    })
  }

  set userID(id: string) {
    if (id == this._userID) return
    this._userID = id
    this._userName = ''
    this._callerName = ''
    this._conversations = []
    this._prevConversationCursor = undefined
    this._latestConversationId = 0
    this._isLoading = false
    this._isLoadingPrev = false
    this._isThinking = 0
    this._isSubmitting = false
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

    if (res.output.error) {
      console.error('GetConversation', res.output.error)
      throw res.output.error
    }

    const now_ms = Date.now()
    const conversation = res.output.result!
    this.addConversation(conversation)
    if (this._isThinking > 0) {
      this._isThinking = isThinking(conversation)
    }

    if (
      (conversation.status == 'submitted' ||
        conversation.status == 'working') &&
      now_ms - conversation.updated_at < 1200000
    ) {
      sleep(Math.min(interval, max_interval)).then(() => {
        this.fetchConversation(_id, Math.floor(interval * 1.1), max_interval)
      })
    }
  }

  async listKipLogs(cursor?: string, limit?: number) {
    const res: ToolOutput<Response<KIPLog[]>> = await tool_call({
      name: 'memory_api',
      args: {
        _type: 'ListKipLogs',
        cursor,
        limit
      } as MemoryToolArgs
    })

    if (res.output.error) {
      console.error('ListKipLogs', res.output.error)
      throw res.output.error
    }

    return res.output
  }

  async listConversations(cursor?: string, limit?: number) {
    const res: ToolOutput<Response<Conversation[]>> = await tool_call({
      name: 'memory_api',
      args: {
        _type: 'ListPrevConversations',
        cursor,
        limit
      } as MemoryToolArgs
    })

    if (res.output.error) {
      console.error('listConversations', res.output.error)
      throw res.output.error
    }

    return res.output
  }

  async loadPreviousConversations(): Promise<boolean> {
    if (!this._prevConversationCursor || this._isLoadingPrev) return false

    this._isLoadingPrev = true

    let rt = await toastRun(async () => {
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

      if (res.output.error) {
        console.error('loadPreviousConversations', res.output.error)
        throw res.output.error
      }

      const conversations = res.output.result!
      if (conversations.length > 0) {
        this._conversations = [...conversations, ...this._conversations]
        this._prevConversationCursor = res.output.next_cursor
        return !!this._prevConversationCursor
      }
    }).finally()

    this._isLoadingPrev = false
    return rt ?? false
  }

  async loadLatestConversations() {
    if (this._isLoading) return
    this._isLoading = true

    await toastRun(async () => {
      const res: ToolOutput<Response<Conversation[]>> = await tool_call({
        name: 'memory_api',
        args: {
          _type: 'ListPrevConversations',
          limit: 20
        } as MemoryToolArgs
      })

      if (res.output.error) {
        console.error('loadLatestConversations', res.output.error)
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
    }).finally()
    this._isLoading = false
  }

  async chat(content: string, resources?: Resource[]) {
    const prompt = content.trim()
    if (!prompt) return

    this._isSubmitting = true
    await toastRun(async () => {
      const res = await agent_run({
        name: 'assistant',
        prompt,
        resources,
        meta: {
          user: this._callerName
            ? this._callerName
            : this._userName || undefined
        }
      })

      if (res.conversation) {
        this._isThinking = res.conversation
        await this.fetchConversation(res.conversation)
      }

      if (res.failed_reason) {
        throw res.failed_reason
      }
    }).finally()
    this._isSubmitting = false
  }

  async stop() {
    const _id = this._isThinking
    if (_id == 0) return
    await toastRun(async () => {
      const res: ToolOutput<Response<Conversation>> = await tool_call({
        name: 'memory_api',
        args: {
          _type: 'StopConversation',
          _id
        } as MemoryToolArgs
      })

      if (res.output.error) {
        console.error('GetConversation', res.output.error)
        throw res.output.error
      }

      const conversation = res.output.result!
      this.addConversation(conversation)
    }).finally()
    this._isThinking = 0
  }
}

export const assistantStore = new AssistantStore()

AssistantStore.init().catch((err) => {
  console.error('Failed to initialize assistant store', err)
})
