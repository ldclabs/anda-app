import type {
  AgentInput,
  AgentOutput,
  Conversation,
  EngineCard,
  MemoryToolArgs,
  Response,
  ToolInput,
  ToolOutput
} from '$lib/types/assistant'

const conversations = new Map<number, Conversation>()
let currentConversationId = 100

// Mock 数据生成函数
function generateMockConversation(
  id: number,
  user_content?: string,
  assistant_content?: string
): Conversation {
  if (!conversations.has(id)) {
    const now = conversations.has(id + 1)
      ? conversations.get(id + 1)!.created_at - 60 * 60 * 24 * 1000
      : Date.now()
    conversations.set(id, {
      _id: id,
      user: 'mock-user-principal',
      thread: `thread-${id}`,
      messages: [
        {
          role: 'user',
          content: user_content ?? `这是第 ${id} 个测试对话的用户消息`,
          name: '用户'
        },
        {
          role: 'assistant',
          content:
            assistant_content ??
            `这是第 ${id} 个测试对话的助手回复。我理解你的问题并为你提供帮助。`,
          name: '助手'
        }
      ],
      resources: [],
      artifacts: [],
      status: Math.random() > 0.8 ? 'working' : 'completed',
      period: Math.floor(now / (3600 * 1000)),
      created_at: now,
      updated_at: now + 1000
    })
  }

  if (currentConversationId < id) {
    currentConversationId = id
  }

  return conversations.get(id)!
}

function generateMockEngineCard(): EngineCard {
  return {
    id: 'mock-engine-principal',
    info: {
      handle: 'mock-assistant',
      handle_canister: 'mock-canister-principal',
      name: 'Mock Assistant',
      description: '用于本地开发的模拟助手',
      endpoint: 'http://localhost:3000/api',
      protocols: {
        'ANDA': 'http://localhost:3000/.well-known/agents/mock',
        'A2A': 'http://localhost:3000/.well-known/agent.json'
      },
      payments: ['X402']
    },
    agents: [
      {
        definition: {
          name: 'assistant',
          description: '通用助手代理',
          parameters: {
            type: 'object',
            properties: {
              prompt: { type: 'string' },
              resources: { type: 'array' }
            }
          }
        },
        supported_resource_tags: ['text', 'image', 'audio']
      }
    ],
    tools: [
      {
        definition: {
          name: 'memory_api',
          description: '会话记忆管理工具',
          parameters: {
            type: 'object',
            properties: {
              _type: { type: 'string' },
              _id: { type: 'number' },
              cursor: { type: 'string' },
              limit: { type: 'number' }
            }
          }
        },
        supported_resource_tags: ['text']
      }
    ]
  }
}

export async function assistant_info(): Promise<EngineCard> {
  await new Promise((resolve) => setTimeout(resolve, 200))
  return generateMockEngineCard()
}

export async function tool_call<I, O>(
  input: ToolInput<I>
): Promise<ToolOutput<O>> {
  // 模拟网络延迟
  await new Promise((resolve) => setTimeout(resolve, 300))

  if (input.name === 'memory_api') {
    const args = input.args as MemoryToolArgs

    if (args._type === 'ListPrevConversations') {
      const cursorNum = args.cursor
        ? parseInt(args.cursor)
        : currentConversationId + 1
      const limit = Math.min(args.limit || 10, cursorNum)

      const conversations = Array.from({ length: limit }, (_, i) =>
        generateMockConversation(cursorNum - i - 1)
      )
      conversations.reverse()

      const next_cursor =
        conversations[0]._id > 0 ? String(conversations[0]._id) : undefined

      return {
        output: {
          result: conversations,
          next_cursor
        } as Response<Conversation[]>,
        usage: {
          input_tokens: 10,
          output_tokens: 50,
          requests: 1
        }
      } as ToolOutput<O>
    }

    if (args._type === 'GetConversation') {
      const conversation = generateMockConversation(args._id)

      return {
        output: {
          result: conversation
        } as Response<Conversation>,
        usage: {
          input_tokens: 5,
          output_tokens: 25,
          requests: 1
        }
      } as ToolOutput<O>
    }
  }

  // 默认返回
  return {
    output: { result: null } as Response<null>,
    usage: {
      input_tokens: 0,
      output_tokens: 0,
      requests: 1
    }
  } as ToolOutput<O>
}

export async function agent_run(input: AgentInput): Promise<AgentOutput> {
  await new Promise((resolve) => setTimeout(resolve, 1000))

  const responses = [
    '我理解你的问题。让我为你提供一个详细的解答...',
    '这是一个很好的问题！基于我的分析，我建议...',
    '根据你提供的信息，我可以帮你解决这个问题。首先...',
    '让我来帮你分析一下这个情况。从几个方面来看...',
    '我明白你的需求。这里有几个解决方案供你参考...'
  ]

  const randomResponse = responses[Math.floor(Math.random() * responses.length)]
  const conversationId = currentConversationId + 1

  generateMockConversation(conversationId, input.prompt, randomResponse)
  return {
    content: randomResponse,
    usage: {
      input_tokens: input.prompt.length,
      output_tokens: randomResponse.length,
      requests: 1
    },
    conversation: conversationId
  }
}
