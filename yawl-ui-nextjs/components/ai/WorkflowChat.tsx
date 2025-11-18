/**
 * AI Workflow Assistant Component
 * Chat interface for AI-powered workflow assistance
 * Uses Vercel AI SDK elements
 */

'use client'

import React, { useRef, useEffect } from 'react'
import { useAIWorkflowAssistant } from '@/hooks/useAIWorkflowAssistant'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Send, Loader2, Sparkles } from 'lucide-react'

interface WorkflowChatProps {
  onWorkflowGenerated?: (spec: any) => void
  defaultPrompt?: string
}

/**
 * AI Chat interface for workflow assistance
 */
export function WorkflowChat({ onWorkflowGenerated, defaultPrompt }: WorkflowChatProps) {
  const {
    messages,
    input,
    handleInputChange,
    handleSubmit,
    isGenerating,
    suggestedWorkflow,
    suggestions,
  } = useAIWorkflowAssistant()

  const messagesEndRef = useRef<HTMLDivElement>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  useEffect(() => {
    if (suggestedWorkflow) {
      onWorkflowGenerated?.(suggestedWorkflow)
    }
  }, [suggestedWorkflow, onWorkflowGenerated])

  const handleFormSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    handleSubmit(e)
  }

  return (
    <Card className="w-full h-screen flex flex-col">
      <CardHeader>
        <div className="flex items-center gap-2">
          <Sparkles className="h-5 w-5" />
          <CardTitle>AI Workflow Assistant</CardTitle>
        </div>
      </CardHeader>

      <CardContent className="flex-1 flex flex-col overflow-hidden">
        {/* Messages */}
        <div className="flex-1 overflow-y-auto space-y-4 mb-4">
          {messages.length === 0 && (
            <div className="text-center text-muted-foreground py-8">
              <p className="font-medium">Welcome to AI Workflow Assistant</p>
              <p className="text-sm mt-2">
                Describe your workflow and I'll help you design it
              </p>
            </div>
          )}

          {messages.map((message, idx) => (
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
                <p className="text-sm whitespace-pre-wrap">{message.content}</p>
              </div>
            </div>
          ))}

          {isGenerating && (
            <div className="flex justify-start">
              <div className="bg-muted px-4 py-2 rounded-lg flex items-center gap-2">
                <Loader2 className="h-4 w-4 animate-spin" />
                <span className="text-sm">Generating...</span>
              </div>
            </div>
          )}

          {suggestions.length > 0 && (
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
              <p className="font-semibold text-sm mb-2">Suggestions:</p>
              <ul className="space-y-1">
                {suggestions.map((suggestion, idx) => (
                  <li key={idx} className="text-sm text-blue-900">
                    • {suggestion}
                  </li>
                ))}
              </ul>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        {/* Input */}
        <form onSubmit={handleFormSubmit} className="flex gap-2">
          <input
            type="text"
            placeholder="Describe your workflow..."
            value={input}
            onChange={handleInputChange}
            className="flex-1 px-3 py-2 border border-input rounded-md bg-background"
            disabled={isGenerating}
          />
          <Button
            type="submit"
            disabled={isGenerating}
            className="px-4"
          >
            {isGenerating ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Send className="h-4 w-4" />
            )}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}

export default WorkflowChat
