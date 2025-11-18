/**
 * Workflow Chat Component
 * AI-powered chat interface for workflow design assistance
 */

'use client'

import React, { useState } from 'react'
import { useAIWorkflowAssistant } from '@/hooks/useAIWorkflowAssistant'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Send, Sparkles } from 'lucide-react'
import type { YAWLSpecification } from '@/types/yawl'

interface WorkflowChatProps {
  onWorkflowGenerated?: (workflow: YAWLSpecification) => void
  defaultPrompt?: string
}

/**
 * Chat interface for workflow generation
 */
export function WorkflowChat({
  onWorkflowGenerated,
  defaultPrompt,
}: WorkflowChatProps) {
  const { isGenerating, suggestedWorkflow, analysisResult, generateWorkflow } =
    useAIWorkflowAssistant()
  const [input, setInput] = useState(defaultPrompt || '')
  const [messages, setMessages] = useState<Array<{role: string, content: string}>>([
    {
      role: 'assistant',
      content: 'Hello! I\'m your AI workflow assistant. Describe the workflow you need, and I\'ll help you design it.',
    },
  ])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!input.trim()) return

    // Add user message
    setMessages((prev) => [...prev, { role: 'user', content: input }])
    const userInput = input
    setInput('')

    // Generate workflow
    await generateWorkflow(userInput)

    // Add assistant message
    setMessages((prev) => [
      ...prev,
      {
        role: 'assistant',
        content: analysisResult || 'Processing your request...',
      },
    ])
  }

  return (
    <Card className="w-full flex flex-col" style={{ height: '600px' }}>
      <CardHeader className="pb-2">
        <div className="flex items-center gap-2">
          <Sparkles className="h-5 w-5" />
          <CardTitle>AI Workflow Assistant</CardTitle>
        </div>
      </CardHeader>

      <CardContent className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-sm text-muted-foreground">
              Describe your workflow and I'll help you design it
            </p>
          </div>
        ) : (
          messages.map((message: any, idx: number) => (
            <div
              key={idx}
              className={`flex ${
                message.role === 'user' ? 'justify-end' : 'justify-start'
              }`}
            >
              <div
                className={`max-w-md px-4 py-2 rounded-lg ${
                  message.role === 'user'
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted text-muted-foreground'
                }`}
              >
                <p className="text-sm">{message.content}</p>
              </div>
            </div>
          ))
        )}
        {isGenerating && (
          <div className="flex justify-start">
            <div className="bg-muted text-muted-foreground px-4 py-2 rounded-lg">
              <div className="flex gap-1">
                <div className="w-2 h-2 bg-current rounded-full animate-bounce" />
                <div className="w-2 h-2 bg-current rounded-full animate-bounce delay-100" />
                <div className="w-2 h-2 bg-current rounded-full animate-bounce delay-200" />
              </div>
            </div>
          </div>
        )}
      </CardContent>

      <div className="border-t p-4">
        <form onSubmit={handleSubmit} className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Describe your workflow..."
            disabled={isGenerating}
            className="flex-1 px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary disabled:opacity-50"
          />
          <Button
            type="submit"
            disabled={isGenerating || !input.trim()}
            size="sm"
          >
            <Send className="h-4 w-4" />
          </Button>
        </form>
      </div>
    </Card>
  )
}

export default WorkflowChat
