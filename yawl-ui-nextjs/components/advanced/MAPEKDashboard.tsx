/**
 * MAPE-K Autonomous Feedback Loop Dashboard
 * Displays monitoring, analysis, planning, execution, and knowledge
 * Aligned with DOCTRINE MAPE-K principle
 */

'use client'

import React, { useEffect, useState } from 'react'
import { useMAPEK } from '@/hooks/useMAPEK'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Activity, Zap, Brain, BarChart3 } from 'lucide-react'

interface MAPEKDashboardProps {
  workflowId: string
}

/**
 * Real-time MAPE-K dashboard for autonomous workflow adaptation
 */
export function MAPEKDashboard({ workflowId }: MAPEKDashboardProps) {
  const { monitoring, analysis, plan, execution, knowledge, cycleCount } =
    useMAPEK(workflowId)
  const [chartData, setChartData] = useState<Record<string, number[]>>({})

  // Update metrics history for visualization
  useEffect(() => {
    setChartData((prev) => ({
      ...prev,
      cpuUsage: [
        ...(prev.cpuUsage || []).slice(-9),
        monitoring.metrics.cpuUsage || 0,
      ],
      errorRate: [
        ...(prev.errorRate || []).slice(-9),
        monitoring.metrics.errorRate || 0,
      ],
    }))
  }, [monitoring.metrics])

  return (
    <div className="space-y-4 w-full">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg">MAPE-K Autonomic Loop</CardTitle>
            <Badge variant="outline">Cycle {cycleCount}</Badge>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Monitoring */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="border rounded-lg p-3">
              <div className="flex items-center gap-2 mb-2">
                <Activity className="h-4 w-4" />
                <h4 className="font-semibold text-sm">Monitoring</h4>
              </div>
              <div className="space-y-1 text-sm">
                <p>
                  CPU: <span className="font-medium">{monitoring.metrics.cpuUsage?.toFixed(1)}%</span>
                </p>
                <p>
                  Memory:{' '}
                  <span className="font-medium">{monitoring.metrics.memoryUsage?.toFixed(1)}%</span>
                </p>
                <p>
                  Error Rate:{' '}
                  <span className="font-medium">{monitoring.metrics.errorRate?.toFixed(2)}%</span>
                </p>
                <p>
                  Anomalies:{' '}
                  <Badge
                    variant={
                      monitoring.anomalies.length > 0 ? 'destructive' : 'secondary'
                    }
                  >
                    {monitoring.anomalies.length}
                  </Badge>
                </p>
              </div>
            </div>

            {/* Analysis */}
            <div className="border rounded-lg p-3">
              <div className="flex items-center gap-2 mb-2">
                <Brain className="h-4 w-4" />
                <h4 className="font-semibold text-sm">Analysis</h4>
              </div>
              <div className="space-y-1 text-sm">
                <p>
                  Status:{' '}
                  <Badge
                    variant={analysis.isHealthy ? 'secondary' : 'destructive'}
                  >
                    {analysis.isHealthy ? 'Healthy' : 'Degraded'}
                  </Badge>
                </p>
                <p>
                  Confidence:{' '}
                  <span className="font-medium">
                    {(analysis.confidence * 100).toFixed(0)}%
                  </span>
                </p>
                <p>
                  Deviations: <span className="font-medium">{analysis.deviations.length}</span>
                </p>
                <p>
                  Root Causes:{' '}
                  <span className="font-medium">{analysis.rootCauses.length}</span>
                </p>
              </div>
            </div>
          </div>

          {/* Planning & Execution */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="border rounded-lg p-3">
              <div className="flex items-center gap-2 mb-2">
                <Zap className="h-4 w-4" />
                <h4 className="font-semibold text-sm">Planning</h4>
              </div>
              <div className="space-y-1 text-sm">
                <p>
                  Plan Status:{' '}
                  <Badge variant={plan ? 'secondary' : 'outline'}>
                    {plan ? 'Ready' : 'Idle'}
                  </Badge>
                </p>
                {plan && (
                  <>
                    <p>
                      Actions: <span className="font-medium">{plan.actions.length}</span>
                    </p>
                    <p>
                      Priority:{' '}
                      <span className="font-medium">{plan.priority}</span>
                    </p>
                  </>
                )}
              </div>
            </div>

            <div className="border rounded-lg p-3">
              <div className="flex items-center gap-2 mb-2">
                <BarChart3 className="h-4 w-4" />
                <h4 className="font-semibold text-sm">Execution</h4>
              </div>
              <div className="space-y-1 text-sm">
                <p>
                  Status:{' '}
                  <Badge
                    variant={
                      execution.isExecuting ? 'secondary' : 'outline'
                    }
                  >
                    {execution.isExecuting ? 'Running' : 'Idle'}
                  </Badge>
                </p>
                <div className="mt-2">
                  <div className="flex justify-between items-center mb-1">
                    <span className="text-xs font-medium">Progress</span>
                    <span className="text-xs">{execution.progress.toFixed(0)}%</span>
                  </div>
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div
                      className="bg-blue-600 h-2 rounded-full transition-all"
                      style={{ width: `${execution.progress}%` }}
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Knowledge Base */}
          {knowledge.pastAdaptations.length > 0 && (
            <div className="border rounded-lg p-3">
              <h4 className="font-semibold text-sm mb-2">Knowledge Base</h4>
              <div className="space-y-1 text-sm">
                <p>
                  Past Adaptations:{' '}
                  <span className="font-medium">{knowledge.pastAdaptations.length}</span>
                </p>
                <p>
                  Success Rate:{' '}
                  <span className="font-medium">
                    {(
                      (knowledge.pastAdaptations.filter(
                        (a) => a.outcome === 'success'
                      ).length /
                        knowledge.pastAdaptations.length) *
                      100
                    ).toFixed(0)}%
                  </span>
                </p>
              </div>
            </div>
          )}

          {/* Anomalies */}
          {monitoring.anomalies.length > 0 && (
            <div className="bg-red-50 border border-red-200 rounded-lg p-3">
              <h4 className="font-semibold text-sm text-red-900 mb-2">
                Active Anomalies
              </h4>
              <div className="space-y-1">
                {monitoring.anomalies.map((anomaly) => (
                  <div key={anomaly.id} className="text-sm text-red-800">
                    <p className="font-medium">{anomaly.message}</p>
                    <p className="text-xs opacity-75">
                      {anomaly.timestamp.toLocaleTimeString()}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

export default MAPEKDashboard
