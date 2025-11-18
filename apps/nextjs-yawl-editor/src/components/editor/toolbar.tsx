/**
 * DOCTRINE ALIGNMENT: Σ (Sum of All Fears - System Integrity)
 * Toolbar - Top toolbar with common workflow actions
 */

'use client';

import { useCallback, useState } from 'react';
import {
  FileText,
  FolderOpen,
  Save,
  Undo,
  Redo,
  FileDown,
  FileUp,
  Play,
  Eye,
  Edit3,
  CheckCircle,
  AlertCircle,
} from 'lucide-react';
import { useWorkflow } from '@/hooks/use-workflow';
import { useValidation } from '@/hooks/use-validation';
import { useTelemetry } from '@/hooks/use-telemetry';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

export interface ToolbarProps {
  onNew?: () => void;
  onOpen?: () => void;
  onSave?: () => void;
  onExport?: (format: 'turtle' | 'yawl') => void;
  onImport?: (format: 'turtle' | 'yawl') => void;
}

/**
 * Toolbar - Top toolbar with common workflow actions
 *
 * Features:
 * - New/Open/Save buttons
 * - Undo/Redo (backed by RDF snapshots)
 * - Export to Turtle / YAWL XML
 * - Import from Turtle / YAWL
 * - Validation status indicator
 * - View mode selector
 *
 * @example
 * ```tsx
 * <Toolbar
 *   onSave={() => saveWorkflow()}
 *   onExport={(format) => exportWorkflow(format)}
 * />
 * ```
 */
export function Toolbar({ onNew, onOpen, onSave, onExport, onImport }: ToolbarProps) {
  const { workflow, mode, setMode, canUndo, canRedo, undo, redo } = useWorkflow();
  const { validation, validate, isValidating } = useValidation();
  const { trackEvent } = useTelemetry('Toolbar');

  const [exportFormat] = useState<'turtle' | 'yawl'>('turtle');
  const [importFormat] = useState<'turtle' | 'yawl'>('turtle');

  const handleNew = useCallback(() => {
    trackEvent('toolbar.new');
    onNew?.();
  }, [onNew, trackEvent]);

  const handleOpen = useCallback(() => {
    trackEvent('toolbar.open');
    onOpen?.();
  }, [onOpen, trackEvent]);

  const handleSave = useCallback(() => {
    trackEvent('toolbar.save');
    onSave?.();
  }, [onSave, trackEvent]);

  const handleUndo = useCallback(() => {
    trackEvent('toolbar.undo');
    undo();
  }, [undo, trackEvent]);

  const handleRedo = useCallback(() => {
    trackEvent('toolbar.redo');
    redo();
  }, [redo, trackEvent]);

  const handleExport = useCallback(() => {
    trackEvent('toolbar.export', { format: exportFormat });
    onExport?.(exportFormat);
  }, [exportFormat, onExport, trackEvent]);

  const handleImport = useCallback(() => {
    trackEvent('toolbar.import', { format: importFormat });
    onImport?.(importFormat);
  }, [importFormat, onImport, trackEvent]);

  const handleValidate = useCallback(async () => {
    trackEvent('toolbar.validate');
    await validate();
  }, [validate, trackEvent]);

  const handleModeChange = useCallback((newMode: 'edit' | 'view' | 'validate') => {
    trackEvent('toolbar.mode', { mode: newMode });
    setMode(newMode);
  }, [setMode, trackEvent]);

  const validationStatus = validation
    ? validation.valid
      ? 'valid'
      : 'invalid'
    : 'unknown';

  const validationIcon = {
    valid: CheckCircle,
    invalid: AlertCircle,
    unknown: Play,
  }[validationStatus];

  const ValidationIcon = validationIcon;

  const validationColor = {
    valid: 'text-green-600',
    invalid: 'text-red-600',
    unknown: 'text-gray-400',
  }[validationStatus];

  return (
    <div className="flex items-center gap-2 p-3 border-b bg-white shadow-sm">
      {/* File operations */}
      <div className="flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          onClick={handleNew}
          title="New Workflow"
        >
          <FileText className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleOpen}
          title="Open Workflow"
        >
          <FolderOpen className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleSave}
          disabled={!workflow}
          title="Save Workflow"
        >
          <Save className="h-4 w-4" />
        </Button>
      </div>

      <Separator orientation="vertical" className="h-6" />

      {/* Edit operations */}
      <div className="flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          onClick={handleUndo}
          disabled={!canUndo}
          title="Undo (Ctrl+Z)"
        >
          <Undo className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleRedo}
          disabled={!canRedo}
          title="Redo (Ctrl+Y)"
        >
          <Redo className="h-4 w-4" />
        </Button>
      </div>

      <Separator orientation="vertical" className="h-6" />

      {/* Import/Export */}
      <div className="flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          onClick={handleImport}
          title={`Import from ${importFormat.toUpperCase()}`}
        >
          <FileUp className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleExport}
          disabled={!workflow}
          title={`Export to ${exportFormat.toUpperCase()}`}
        >
          <FileDown className="h-4 w-4" />
        </Button>
      </div>

      <Separator orientation="vertical" className="h-6" />

      {/* View mode selector */}
      <div className="flex items-center gap-1">
        <Button
          variant={mode === 'edit' ? 'default' : 'ghost'}
          size="sm"
          onClick={() => handleModeChange('edit')}
          title="Edit Mode"
        >
          <Edit3 className="h-4 w-4" />
        </Button>
        <Button
          variant={mode === 'view' ? 'default' : 'ghost'}
          size="sm"
          onClick={() => handleModeChange('view')}
          title="View Mode"
        >
          <Eye className="h-4 w-4" />
        </Button>
        <Button
          variant={mode === 'validate' ? 'default' : 'ghost'}
          size="sm"
          onClick={() => handleModeChange('validate')}
          title="Validate Mode"
        >
          <Play className="h-4 w-4" />
        </Button>
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Validation status */}
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={handleValidate}
          disabled={!workflow || isValidating}
        >
          <Play className="h-4 w-4 mr-2" />
          {isValidating ? 'Validating...' : 'Validate'}
        </Button>

        {validation && (
          <Badge
            variant={validation.valid ? 'default' : 'destructive'}
            className="flex items-center gap-1"
          >
            <ValidationIcon className={cn('h-3 w-3', validationColor)} />
            {validation.valid
              ? 'Valid'
              : `${validation.errors.length} error${validation.errors.length > 1 ? 's' : ''}`}
          </Badge>
        )}
      </div>
    </div>
  );
}
