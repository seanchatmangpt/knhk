'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

export default function DocsPage() {
  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Documentation</h1>
        <p className="text-muted-foreground mt-2">
          Learn about YAWL workflows and how to use this platform
        </p>
      </div>

      <Tabs defaultValue="concepts" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="concepts">Concepts</TabsTrigger>
          <TabsTrigger value="patterns">Patterns</TabsTrigger>
          <TabsTrigger value="rdf">RDF/Turtle</TabsTrigger>
          <TabsTrigger value="api">API</TabsTrigger>
        </TabsList>

        <TabsContent value="concepts" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>YAWL Concepts</CardTitle>
              <CardDescription>
                Fundamental concepts of YAWL workflow language
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <h3 className="font-semibold mb-2">Specification</h3>
                <p className="text-sm text-muted-foreground">
                  A workflow specification defines the structure and behavior of a workflow.
                  It contains tasks, control flows, and data mappings.
                </p>
              </div>
              <div>
                <h3 className="font-semibold mb-2">Task</h3>
                <p className="text-sm text-muted-foreground">
                  An atomic unit of work in a workflow. Tasks can be assigned to humans or
                  executed automatically by services.
                </p>
              </div>
              <div>
                <h3 className="font-semibold mb-2">Case (Instance)</h3>
                <p className="text-sm text-muted-foreground">
                  A running instance of a workflow specification. Each case has its own
                  state, data, and work items.
                </p>
              </div>
              <div>
                <h3 className="font-semibold mb-2">Work Item</h3>
                <p className="text-sm text-muted-foreground">
                  An individual task instance that requires action. Work items go through
                  states: offered, allocated, started, completed.
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="patterns" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Control Flow Patterns</CardTitle>
              <CardDescription>
                Common workflow patterns supported by YAWL
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <h3 className="font-semibold mb-2">Sequence</h3>
                <p className="text-sm text-muted-foreground">
                  Tasks are executed one after another in order.
                </p>
              </div>
              <div>
                <h3 className="font-semibold mb-2">Parallel Split</h3>
                <p className="text-sm text-muted-foreground">
                  Multiple tasks execute simultaneously, all must start at the same time.
                </p>
              </div>
              <div>
                <h3 className="font-semibold mb-2">Exclusive Choice</h3>
                <p className="text-sm text-muted-foreground">
                  One of several paths is chosen based on conditions. Only one path executes.
                </p>
              </div>
              <div>
                <h3 className="font-semibold mb-2">Synchronization</h3>
                <p className="text-sm text-muted-foreground">
                  Waits for multiple parallel tasks to complete before continuing.
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="rdf" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>RDF/Turtle Support</CardTitle>
              <CardDescription>
                Representing YAWL workflows in Semantic Web format
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <h3 className="font-semibold mb-2 font-mono text-sm">Turtle Format</h3>
                <p className="text-sm text-muted-foreground">
                  YAWL workflows can be serialized to Turtle RDF format for semantic
                  interoperability and linked data integration.
                </p>
              </div>
              <div>
                <h3 className="font-semibold mb-2">Import/Export</h3>
                <p className="text-sm text-muted-foreground">
                  You can import workflows from Turtle files and export your workflows
                  in RDF format for external processing.
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="api" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>API Reference</CardTitle>
              <CardDescription>
                REST API endpoints for workflow management
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <h3 className="font-semibold mb-2 font-mono text-sm">GET /api/specifications</h3>
                <p className="text-sm text-muted-foreground">
                  List all workflow specifications
                </p>
              </div>
              <div>
                <h3 className="font-semibold mb-2 font-mono text-sm">POST /api/specifications</h3>
                <p className="text-sm text-muted-foreground">
                  Create a new workflow specification
                </p>
              </div>
              <div>
                <h3 className="font-semibold mb-2 font-mono text-sm">GET /api/cases</h3>
                <p className="text-sm text-muted-foreground">
                  List all workflow cases
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
