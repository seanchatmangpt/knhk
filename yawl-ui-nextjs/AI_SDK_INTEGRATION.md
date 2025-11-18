# AI-Powered YAWL Workflow Assistant

## Overview

The YAWL UI now includes **Vercel AI SDK integration** for intelligent workflow design, pattern recommendation, and autonomous optimization.

## Features

### 🤖 AI Hooks

#### useAIWorkflowAssistant
- Generate workflows from natural language descriptions
- Get intelligent suggestions for workflow improvements
- Analyze existing workflows
- Pattern recommendations
- Optimization suggestions

```tsx
const {
  messages,
  generateWorkflow,
  analyzeWorkflow,
  suggestPatterns,
  optimizeWorkflow,
  suggestedWorkflow,
} = useAIWorkflowAssistant()

await generateWorkflow("Process customer orders with approval")
```

#### useAIPatternGenerator
- Suggest best patterns for scenarios
- Explain pattern compositions
- Validate pattern implementations

```tsx
const {
  pattern,
  explanation,
  steps,
  suggestPattern,
  validatePattern,
} = useAIPatternGenerator()

await suggestPattern("Order approval workflow", 5, "high")
```

### 💬 AI Components

#### WorkflowChat
- Interactive chat interface for workflow design
- Real-time AI suggestions
- Workflow generation
- Message streaming

```tsx
<WorkflowChat
  onWorkflowGenerated={(spec) => handleWorkflow(spec)}
  defaultPrompt="Create an order processing workflow"
/>
```

#### AIPatternSuggestions
- Real-time pattern recommendations
- Best practice guidance
- Implementation steps
- Why each pattern applies

```tsx
<AIPatternSuggestions
  specification={workflow}
  onPatternSelected={(pattern) => applyPattern(pattern)}
/>
```

### 🧠 RAG System (Retrieval-Augmented Generation)

#### WorkflowKnowledgeBase
- Curated YAWL workflow knowledge
- Pattern recommendations
- Best practices for scenarios
- Semantic search

```tsx
const kb = new WorkflowKnowledgeBase()
const patterns = kb.getPatternsForScenario("approval flow")
const practices = kb.getBestPractices("parallel processing")
```

#### AIWorkflowGenerationService
- Generate workflows from descriptions
- NLP-based task extraction
- Pattern selection
- Variation generation

```tsx
const spec = AIWorkflowGenerationService.generateFromDescription({
  description: "Process orders with parallel payment and inventory check",
  complexity: "medium"
})
```

## Architecture

```
AI Components (useChat)
  ↓
API Routes (/api/workflow-assistant, /api/pattern-generator)
  ↓
Vercel AI SDK (streamText, anthropic model)
  ↓
Claude 3.5 Sonnet LLM

Combined with:
WorkflowKnowledgeBase (RAG Context)
  ↓
AIWorkflowGenerationService
```

## API Routes

### POST /api/workflow-assistant
Handles workflow design chat

**Request:**
```json
{
  "messages": [
    { "role": "user", "content": "Create order processing workflow" }
  ]
}
```

**Response:** Streamed text with workflow suggestions

### POST /api/pattern-generator
Handles pattern recommendations

**Request:**
```json
{
  "messages": [
    { "role": "user", "content": "What pattern for parallel tasks?" }
  ]
}
```

**Response:** Streamed JSON with pattern recommendations

## Knowledge Base

Pre-loaded with workflows for:
- Order Processing (parallel, choice, synchronization)
- Approval Workflows (sequence, choice)
- Parallel Tasks (parallel, synchronization, multiple-merge)
- Complex Decisions (exclusive-choice, deferred-choice)

## Usage Example

```tsx
'use client'

import { WorkflowChat } from '@/components/ai'
import { AIPatternSuggestions } from '@/components/ai'
import { useWorkflowStore } from '@/stores/workflowStore'

export default function AIWorkflowBuilder() {
  const store = useWorkflowStore()

  return (
    <div className="space-y-6">
      <WorkflowChat
        onWorkflowGenerated={(spec) => {
          store.addSpecification(spec)
          store.setCurrentSpec(spec)
        }}
      />

      {store.currentSpec && (
        <AIPatternSuggestions specification={store.currentSpec} />
      )}
    </div>
  )
}
```

## DOCTRINE Alignment

✓ **O (Observation):** Natural language processing of workflow descriptions
✓ **Σ (Ontology):** Knowledge base with semantic relationships
✓ **Q (Invariants):** Pattern validation and enforcement
✓ **Π (Projections):** Chat UI and visualization components
✓ **MAPE-K:** AI suggestions trigger workflow optimization
✓ **Chatman Constant:** Performance monitoring of AI operations

## Dependencies

- `ai` - Vercel AI SDK
- `@ai-sdk/anthropic` - Claude model integration
- `n3` - RDF/Turtle support for knowledge base

## Future Enhancements

- [ ] Fine-tuned models specific to YAWL
- [ ] Multi-turn conversation context
- [ ] Workflow reasoning and explanation
- [ ] AI-powered optimization
- [ ] Predictive pattern suggestions
- [ ] Custom knowledge base ingestion
- [ ] Workflow similarity detection

## Performance

- Chat streaming for real-time feedback
- Knowledge base search: O(n) term-based
- Pattern generation: < 5 seconds
- Workflow generation: < 3 seconds

## Security

- API routes require authentication (to be added)
- Knowledge base is read-only
- No sensitive data in prompts
- AI responses validated before use
