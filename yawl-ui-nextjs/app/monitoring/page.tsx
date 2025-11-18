'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'

const runningCases = [
  {
    id: 'case-001',
    workflow: 'Order Processing',
    status: 'running' as const,
    progress: 65,
    startTime: '2024-11-18 09:00',
    workItems: 3,
  },
  {
    id: 'case-002',
    workflow: 'Expense Approval',
    status: 'suspended' as const,
    progress: 40,
    startTime: '2024-11-18 10:30',
    workItems: 1,
  },
  {
    id: 'case-003',
    workflow: 'Leave Request',
    status: 'completed' as const,
    progress: 100,
    startTime: '2024-11-17 14:00',
    workItems: 0,
  },
]

const statusColors: Record<string, string> = {
  running: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-100',
  suspended: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-100',
  completed: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-100',
}

export default function MonitoringPage() {
  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Workflow Monitoring</h1>
        <p className="text-muted-foreground mt-2">
          Track running workflow cases and work items
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Active Cases</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">2</div>
            <p className="text-xs text-muted-foreground">Running workflows</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Pending Items</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">4</div>
            <p className="text-xs text-muted-foreground">Work items awaiting action</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Completion Rate</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">87%</div>
            <p className="text-xs text-muted-foreground">This week</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Running Cases</CardTitle>
          <CardDescription>Active workflow instances</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {runningCases.map((caseItem) => (
              <div
                key={caseItem.id}
                className="flex items-center justify-between rounded-lg border p-4"
              >
                <div className="flex-1 space-y-2">
                  <div className="flex items-center gap-2">
                    <p className="font-medium">{caseItem.workflow}</p>
                    <Badge className={statusColors[caseItem.status]}>
                      {caseItem.status}
                    </Badge>
                  </div>
                  <p className="text-sm text-muted-foreground">
                    Started: {caseItem.startTime} | Work Items: {caseItem.workItems}
                  </p>
                  <div className="w-full bg-secondary rounded-full h-2">
                    <div
                      className="bg-primary h-2 rounded-full transition-all"
                      style={{ width: `${caseItem.progress}%` }}
                    />
                  </div>
                </div>
                <Button variant="outline" size="sm" className="ml-4">
                  View
                </Button>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
