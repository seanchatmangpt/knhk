'use client'

import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Plus, Download, Upload, RotateCcw } from 'lucide-react'
import WorkflowService from '@/lib/workflow-service'
import type { YAWLSpecification } from '@/types/yawl'

export default function EditorPage() {
  const [workflow, setWorkflow] = useState<YAWLSpecification | null>(null)
  const [taskName, setTaskName] = useState('')

  const createNewWorkflow = () => {
    const newWorkflow = WorkflowService.createSpecification('wf-001', 'New Workflow')
    setWorkflow(newWorkflow)
  }

  const addTask = () => {
    if (!workflow || !taskName) return

    const newTask = {
      id: `task-${Date.now()}`,
      name: taskName,
      type: 'atomic' as const,
      documentation: `Task: ${taskName}`,
    }

    const updated = WorkflowService.addTask(workflow, newTask)
    setWorkflow(updated)
    setTaskName('')
  }

  const validateWorkflow = () => {
    if (!workflow) return
    const result = WorkflowService.validate(workflow)
    console.log('Validation Result:', result)
    alert(`Validation: ${result.isValid ? 'Valid' : 'Invalid'}\nErrors: ${result.errors.length}\nWarnings: ${result.warnings.length}`)
  }

  const downloadWorkflow = () => {
    if (!workflow) return
    const json = JSON.stringify(workflow, null, 2)
    const blob = new Blob([json], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${workflow.id}.json`
    a.click()
  }

  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Workflow Editor</h1>
        <p className="text-muted-foreground mt-2">Create and edit YAWL workflows</p>
      </div>

      {!workflow ? (
        <Card>
          <CardHeader>
            <CardTitle>Create New Workflow</CardTitle>
            <CardDescription>Start by creating a new workflow specification</CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={createNewWorkflow} className="w-full">
              <Plus className="mr-2 h-4 w-4" />
              New Workflow
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle>{workflow.name}</CardTitle>
                  <CardDescription>ID: {workflow.id} | Version: {workflow.version}</CardDescription>
                </div>
                <div className="space-x-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={validateWorkflow}
                  >
                    Validate
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={downloadWorkflow}
                  >
                    <Download className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setWorkflow(null)}
                  >
                    <RotateCcw className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </CardHeader>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Add Task</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder="Task name..."
                  value={taskName}
                  onChange={(e) => setTaskName(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && addTask()}
                  className="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm"
                />
                <Button onClick={addTask}>
                  <Plus className="mr-2 h-4 w-4" />
                  Add Task
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Tasks ({workflow.tasks.length})</CardTitle>
            </CardHeader>
            <CardContent>
              {workflow.tasks.length === 0 ? (
                <p className="text-sm text-muted-foreground">No tasks yet. Add one to get started.</p>
              ) : (
                <div className="space-y-2">
                  {workflow.tasks.map((task) => (
                    <div
                      key={task.id}
                      className="flex items-center justify-between rounded-md border p-3"
                    >
                      <div>
                        <p className="font-medium">{task.name}</p>
                        <p className="text-xs text-muted-foreground">{task.id}</p>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                          const updated = WorkflowService.removeTask(workflow, task.id)
                          setWorkflow(updated)
                        }}
                      >
                        Remove
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  )
}
