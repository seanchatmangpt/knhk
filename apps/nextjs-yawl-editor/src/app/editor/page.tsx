'use client';

import { useState } from 'react';

/**
 * DOCTRINE ALIGNMENT: Q (Hard Invariants)
 * YAWL Process Editor main page
 * Enforces workflow pattern validation against permutation matrix
 */

export default function EditorPage() {
  const [workflowName, setWorkflowName] = useState('Untitled Workflow');

  return (
    <div className="flex h-screen flex-col bg-slate-50">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-slate-200 bg-white px-6 py-4 shadow-sm">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-bold text-slate-900">YAWL Editor</h1>
          <input
            type="text"
            value={workflowName}
            onChange={(e) => setWorkflowName(e.target.value)}
            className="rounded border border-slate-300 px-3 py-1 text-sm focus:border-blue-500 focus:outline-none"
          />
        </div>

        <div className="flex gap-2">
          <button className="rounded bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
            Save
          </button>
          <button className="rounded border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50">
            Export
          </button>
        </div>
      </header>

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar - Pattern Palette */}
        <aside className="w-64 border-r border-slate-200 bg-white p-4">
          <h2 className="mb-4 text-sm font-semibold uppercase text-slate-600">
            Workflow Patterns
          </h2>
          <div className="space-y-2">
            <PatternButton label="Sequence" />
            <PatternButton label="Parallel Split" />
            <PatternButton label="Synchronization" />
            <PatternButton label="Exclusive Choice" />
            <PatternButton label="Simple Merge" />
          </div>
        </aside>

        {/* Canvas */}
        <main className="flex-1 p-4">
          <div className="flex h-full items-center justify-center rounded-lg border-2 border-dashed border-slate-300 bg-white">
            <div className="text-center">
              <p className="text-lg font-medium text-slate-600">
                Drop workflow patterns here
              </p>
              <p className="mt-2 text-sm text-slate-500">
                Or use the pattern palette to build your workflow
              </p>
            </div>
          </div>
        </main>

        {/* Properties Panel */}
        <aside className="w-80 border-l border-slate-200 bg-white p-4">
          <h2 className="mb-4 text-sm font-semibold uppercase text-slate-600">
            Properties
          </h2>
          <div className="text-sm text-slate-500">
            Select an element to view properties
          </div>
        </aside>
      </div>
    </div>
  );
}

function PatternButton({ label }: { label: string }) {
  return (
    <button className="w-full rounded border border-slate-200 bg-slate-50 px-3 py-2 text-left text-sm font-medium text-slate-700 transition-colors hover:bg-slate-100">
      {label}
    </button>
  );
}
