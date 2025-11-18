import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import Link from 'next/link'

export default function Home() {
  return (
    <div className="space-y-8 p-6">
      <div className="space-y-2">
        <h1 className="text-4xl font-bold tracking-tight">YAWL Workflow Management</h1>
        <p className="text-lg text-muted-foreground">
          A modern Next.js implementation of YAWL (Yet Another Workflow Language) with RDF support
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Workflow Editor</CardTitle>
            <CardDescription>Design and visualize workflows</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="mb-4 text-sm text-muted-foreground">
              Create, edit, and validate YAWL workflow specifications with an interactive editor.
            </p>
            <Link href="/editor">
              <Button>Open Editor</Button>
            </Link>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Workflow Library</CardTitle>
            <CardDescription>Browse workflow templates</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="mb-4 text-sm text-muted-foreground">
              Explore pre-built workflows and patterns from the YAWL repository.
            </p>
            <Link href="/workflows">
              <Button>Browse Workflows</Button>
            </Link>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>RDF Import/Export</CardTitle>
            <CardDescription>Work with Turtle files</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="mb-4 text-sm text-muted-foreground">
              Import and export workflows in RDF/Turtle format for semantic integration.
            </p>
            <Link href="/import-export">
              <Button>Manage RDF</Button>
            </Link>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Pattern Validation</CardTitle>
            <CardDescription>Validate workflow patterns</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="mb-4 text-sm text-muted-foreground">
              Ensure workflows conform to YAWL patterns and control flow rules.
            </p>
            <Link href="/validation">
              <Button>Validate Patterns</Button>
            </Link>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Execution Monitoring</CardTitle>
            <CardDescription>Track workflow execution</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="mb-4 text-sm text-muted-foreground">
              Monitor running workflow cases and work items in real-time.
            </p>
            <Link href="/monitoring">
              <Button>Monitor Cases</Button>
            </Link>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Documentation</CardTitle>
            <CardDescription>Learn YAWL concepts</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="mb-4 text-sm text-muted-foreground">
              Read comprehensive guides on YAWL patterns, RDF semantics, and best practices.
            </p>
            <Link href="/docs">
              <Button>View Docs</Button>
            </Link>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
