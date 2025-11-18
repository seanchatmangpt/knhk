import Link from 'next/link';
import { FileCode2, Workflow, Database } from 'lucide-react';

/**
 * DOCTRINE ALIGNMENT: O (Ontology-First)
 * Landing page for YAWL Process Editor
 * Emphasizes semantic modeling and RDF/Turtle ontology integration
 */

export default function HomePage() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center bg-gradient-to-b from-slate-50 to-slate-100 p-24">
      <div className="z-10 w-full max-w-5xl items-center justify-between font-mono text-sm">
        <h1 className="mb-8 text-center text-5xl font-bold tracking-tight text-slate-900">
          YAWL Process Editor
        </h1>

        <p className="mb-12 text-center text-lg text-slate-600">
          Next.js-based workflow editor with RDF/Turtle ontology support
        </p>

        <div className="mb-12 grid gap-6 md:grid-cols-3">
          <FeatureCard
            icon={<Workflow className="h-8 w-8 text-blue-600" />}
            title="Workflow Modeling"
            description="Visual YAWL workflow pattern editor with 43 control flow patterns"
          />

          <FeatureCard
            icon={<FileCode2 className="h-8 w-8 text-green-600" />}
            title="RDF/Turtle Support"
            description="Native ontology integration with semantic validation"
          />

          <FeatureCard
            icon={<Database className="h-8 w-8 text-purple-600" />}
            title="Pattern Validation"
            description="Real-time validation against YAWL pattern permutation matrix"
          />
        </div>

        <div className="flex justify-center gap-4">
          <Link
            href="/editor"
            className="rounded-lg bg-blue-600 px-6 py-3 font-semibold text-white transition-colors hover:bg-blue-700"
          >
            Open Editor
          </Link>

          <a
            href="https://github.com/ruvnet/knhk"
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-lg border border-slate-300 bg-white px-6 py-3 font-semibold text-slate-700 transition-colors hover:bg-slate-50"
          >
            Documentation
          </a>
        </div>

        <footer className="mt-16 text-center text-sm text-slate-500">
          <p>Built with Next.js 15, TypeScript, and OpenTelemetry</p>
          <p className="mt-2">
            DOCTRINE 2027 Compliant | Covenant-Driven Development
          </p>
        </footer>
      </div>
    </main>
  );
}

interface FeatureCardProps {
  icon: React.ReactNode;
  title: string;
  description: string;
}

function FeatureCard({ icon, title, description }: FeatureCardProps) {
  return (
    <div className="rounded-lg border border-slate-200 bg-white p-6 shadow-sm transition-shadow hover:shadow-md">
      <div className="mb-4">{icon}</div>
      <h3 className="mb-2 text-xl font-semibold text-slate-900">{title}</h3>
      <p className="text-slate-600">{description}</p>
    </div>
  );
}
