/**
 * API Route: Workflow Assistant Chat
 * Handles AI-powered workflow suggestions using Vercel AI SDK
 */

import { streamText } from 'ai'
import { anthropic } from '@ai-sdk/anthropic'
import { NextRequest, NextResponse } from 'next/server'

const systemPrompt = `You are an expert YAWL workflow designer. Help users create optimal workflows.

When a user asks about workflow design:
1. Understand their business process
2. Suggest appropriate YAWL control flow patterns
3. Provide a JSON workflow structure if requested

Available patterns: sequence, parallel, choice, exclusive-choice, synchronization, multiple-merge, deferred-choice, implicit-choice, interleaved-parallel, multi-choice, discriminator

Always respond with:
- Clear explanation
- Pattern recommendations
- Implementation suggestions

If user asks for JSON workflow, format as:
{
  "workflow": {
    "id": "wf-xxx",
    "name": "Workflow Name",
    "version": "1.0",
    "tasks": [...],
    "nets": [...]
  },
  "suggestions": ["suggestion1", "suggestion2"],
  "analysis": "detailed analysis"
}`

export async function POST(request: NextRequest) {
  const { messages } = await request.json()

  const result = streamText({
    model: anthropic('claude-3-5-sonnet-20241022'),
    system: systemPrompt,
    messages,
  })

  return result.toTextStreamResponse()
}
