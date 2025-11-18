/**
 * DOCTRINE ALIGNMENT: O (Ontology-First)
 * PatternPalette - Draggable pattern elements for workflow construction
 */

'use client';

import { useCallback, useState } from 'react';
import { Square, Diamond, GitBranch, GitMerge, Play, CheckCircle, Search } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { cn } from '@/lib/utils';
import { useTelemetry } from '@/hooks/use-telemetry';

interface PatternItem {
  id: string;
  type: 'task' | 'condition' | 'start' | 'end' | 'split' | 'join';
  label: string;
  description: string;
  icon: React.ComponentType<{ className?: string }>;
  category: 'basic' | 'control' | 'advanced';
}

const PATTERN_ITEMS: PatternItem[] = [
  {
    id: 'start',
    type: 'start',
    label: 'Start',
    description: 'Workflow entry point',
    icon: Play,
    category: 'basic',
  },
  {
    id: 'end',
    type: 'end',
    label: 'End',
    description: 'Workflow exit point',
    icon: CheckCircle,
    category: 'basic',
  },
  {
    id: 'task',
    type: 'task',
    label: 'Task',
    description: 'Atomic work unit',
    icon: Square,
    category: 'basic',
  },
  {
    id: 'condition',
    type: 'condition',
    label: 'Condition',
    description: 'Decision point',
    icon: Diamond,
    category: 'control',
  },
  {
    id: 'split',
    type: 'split',
    label: 'Split',
    description: 'Parallel or exclusive split',
    icon: GitBranch,
    category: 'control',
  },
  {
    id: 'join',
    type: 'join',
    label: 'Join',
    description: 'Synchronize parallel flows',
    icon: GitMerge,
    category: 'control',
  },
];

export interface PatternPaletteProps {
  className?: string;
}

/**
 * PatternPalette - Sidebar with draggable YAWL pattern elements
 *
 * Features:
 * - Pattern palette items
 * - Drag-to-canvas support
 * - Category grouping
 * - Search/filter
 *
 * @example
 * ```tsx
 * <PatternPalette className="w-64" />
 * ```
 */
export function PatternPalette({ className }: PatternPaletteProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const { trackEvent } = useTelemetry('PatternPalette');

  // Filter patterns based on search and category
  const filteredPatterns = PATTERN_ITEMS.filter((pattern) => {
    const matchesSearch = pattern.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
      pattern.description.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesCategory = !selectedCategory || pattern.category === selectedCategory;
    return matchesSearch && matchesCategory;
  });

  // Group patterns by category
  const categorizedPatterns = {
    basic: filteredPatterns.filter((p) => p.category === 'basic'),
    control: filteredPatterns.filter((p) => p.category === 'control'),
    advanced: filteredPatterns.filter((p) => p.category === 'advanced'),
  };

  const handleDragStart = useCallback(
    (event: React.DragEvent<HTMLDivElement>, pattern: PatternItem) => {
      trackEvent('pattern.dragStart', { patternType: pattern.type });
      event.dataTransfer.setData('application/reactflow', pattern.type);
      event.dataTransfer.effectAllowed = 'move';
    },
    [trackEvent]
  );

  const handleDragEnd = useCallback(
    (pattern: PatternItem) => {
      trackEvent('pattern.dragEnd', { patternType: pattern.type });
    },
    [trackEvent]
  );

  const renderPattern = (pattern: PatternItem) => {
    const Icon = pattern.icon;

    return (
      <div
        key={pattern.id}
        draggable
        onDragStart={(e) => handleDragStart(e, pattern)}
        onDragEnd={() => handleDragEnd(pattern)}
        className={cn(
          'flex items-start gap-3 p-3 rounded-lg border-2 border-dashed',
          'cursor-move hover:border-blue-500 hover:bg-blue-50',
          'transition-all duration-150',
          'bg-white'
        )}
      >
        <div className="flex-shrink-0 mt-0.5">
          <div className="w-8 h-8 rounded-md bg-blue-100 flex items-center justify-center">
            <Icon className="h-4 w-4 text-blue-600" />
          </div>
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900">{pattern.label}</div>
          <div className="text-xs text-gray-500 mt-0.5 line-clamp-2">
            {pattern.description}
          </div>
        </div>
      </div>
    );
  };

  return (
    <Card className={cn('border-r rounded-none shadow-lg', className)}>
      <CardHeader className="pb-4">
        <CardTitle className="text-lg">Pattern Palette</CardTitle>
        <CardDescription className="text-xs">
          Drag patterns to canvas
        </CardDescription>

        {/* Search */}
        <div className="pt-4">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
            <Input
              placeholder="Search patterns..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9"
            />
          </div>
        </div>

        {/* Category filters */}
        <div className="flex gap-2 pt-3">
          <Badge
            variant={!selectedCategory ? 'default' : 'outline'}
            className="cursor-pointer"
            onClick={() => setSelectedCategory(null)}
          >
            All
          </Badge>
          <Badge
            variant={selectedCategory === 'basic' ? 'default' : 'outline'}
            className="cursor-pointer"
            onClick={() => setSelectedCategory('basic')}
          >
            Basic
          </Badge>
          <Badge
            variant={selectedCategory === 'control' ? 'default' : 'outline'}
            className="cursor-pointer"
            onClick={() => setSelectedCategory('control')}
          >
            Control
          </Badge>
        </div>
      </CardHeader>

      <Separator />

      <CardContent className="p-0">
        <ScrollArea className="h-[calc(100vh-280px)]">
          <div className="p-4 space-y-4">
            {/* Basic Patterns */}
            {categorizedPatterns.basic.length > 0 && (
              <div className="space-y-2">
                <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide px-1">
                  Basic
                </div>
                <div className="space-y-2">
                  {categorizedPatterns.basic.map(renderPattern)}
                </div>
              </div>
            )}

            {/* Control Flow Patterns */}
            {categorizedPatterns.control.length > 0 && (
              <div className="space-y-2">
                <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide px-1">
                  Control Flow
                </div>
                <div className="space-y-2">
                  {categorizedPatterns.control.map(renderPattern)}
                </div>
              </div>
            )}

            {/* Advanced Patterns */}
            {categorizedPatterns.advanced.length > 0 && (
              <div className="space-y-2">
                <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide px-1">
                  Advanced
                </div>
                <div className="space-y-2">
                  {categorizedPatterns.advanced.map(renderPattern)}
                </div>
              </div>
            )}

            {/* No results */}
            {filteredPatterns.length === 0 && (
              <div className="text-center py-8 text-gray-500 text-sm">
                No patterns found
              </div>
            )}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
