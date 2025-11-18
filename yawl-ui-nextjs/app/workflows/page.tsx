'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import Link from 'next/link'

const sampleWorkflows = [
  {
    id: 'order-process',
    name: 'Order Processing Workflow',
    description: 'Standard order processing with approval and fulfillment',
    tasks: 5,
    version: '1.0',
  },
  {
    id: 'expense-approval',
    name: 'Expense Approval',
    description: 'Employee expense report submission and manager approval',
    tasks: 4,
    version: '1.0',
  },
  {
    id: 'leave-request',
    name: 'Leave Request',
    description: 'Employee leave request with manager and HR approval',
    tasks: 6,
    version: '2.0',
  },
  {
    id: 'recruitment',
    name: 'Recruitment Process',
    description: 'Candidate recruitment with interview and offer stages',
    tasks: 8,
    version: '1.5',
  },
]

export default function WorkflowsPage() {
  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Workflow Library</h1>
        <p className="text-muted-foreground mt-2">
          Browse and use pre-built workflow templates
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {sampleWorkflows.map((workflow) => (
          <Card key={workflow.id}>
            <CardHeader>
              <CardTitle>{workflow.name}</CardTitle>
              <CardDescription>{workflow.description}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex gap-4 text-sm text-muted-foreground">
                <div>Tasks: {workflow.tasks}</div>
                <div>Version: {workflow.version}</div>
              </div>
              <div className="flex gap-2">
                <Button className="flex-1" variant="outline">
                  Preview
                </Button>
                <Button className="flex-1">
                  Use Template
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Create from Scratch</CardTitle>
          <CardDescription>Start building your own workflow</CardDescription>
        </CardHeader>
        <CardContent>
          <Link href="/editor">
            <Button>Open Editor</Button>
          </Link>
        </CardContent>
      </Card>
    </div>
  )
}
