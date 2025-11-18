/**
 * API Route: Pattern Generator Chat
 * Handles AI-powered pattern recommendations
 */

import { streamText } from 'ai'
import { anthropic } from '@ai-sdk/anthropic'
import { NextRequest } from 'next/server'

const systemPrompt = `You are a YAWL workflow pattern expert. Help users select and implement optimal patterns.

When analyzing workflows:
1. Suggest the best control flow pattern
2. Explain why it applies
3. Provide implementation steps
4. Note potential improvements

Respond with JSON:
{
  "pattern": "pattern-name",
  "explanation": "why this pattern is best",
  "steps": ["step1", "step2", ...],
  "benefits": ["benefit1", "benefit2"]
}`

export async function POST(request: NextRequest) {
  const { messages } = await request.json()

  const result = streamText({
    model: anthropic('claude-3-5-sonnet-20241022'),
    system: systemPrompt,
    messages,
  })

  return result.toDataStreamResponse()
}
