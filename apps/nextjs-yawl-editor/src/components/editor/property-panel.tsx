/**
 * DOCTRINE ALIGNMENT: Σ (Sum of All Fears - System Integrity)
 * PropertyPanel - Side panel for editing element properties
 * All changes sync to RDF graph
 */

'use client';

import { useEffect, useState } from 'react';
import { X } from 'lucide-react';
import { useWorkflow } from '@/hooks/use-workflow';
import { useValidation } from '@/hooks/use-validation';
import { useTelemetry } from '@/hooks/use-telemetry';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';
import type { YAWLNode } from '@/lib/types';

export interface PropertyPanelProps {
  selectedNodeId: string | null;
  onClose: () => void;
}

/**
 * PropertyPanel - Edit properties of selected workflow elements
 *
 * Features:
 * - Task properties (name, type, documentation)
 * - Condition properties (name, query)
 * - Real-time validation feedback
 * - Save/Cancel buttons
 * - Sync with RDF store
 *
 * @example
 * ```tsx
 * <PropertyPanel
 *   selectedNodeId={selectedNodeId}
 *   onClose={() => setSelectedNodeId(null)}
 * />
 * ```
 */
export function PropertyPanel({ selectedNodeId, onClose }: PropertyPanelProps) {
  const { workflow, updateNode } = useWorkflow();
  const { validation } = useValidation();
  const { trackEvent } = useTelemetry('PropertyPanel');

  const [formData, setFormData] = useState<Partial<YAWLNode>>({});
  const [hasChanges, setHasChanges] = useState(false);

  // Get selected node
  const selectedNode = workflow?.nodes.find((n) => n.id === selectedNodeId);

  // Get validation errors for this node
  const nodeErrors = validation?.errors.filter((err) => err.node === selectedNodeId) || [];

  // Initialize form data when node changes
  useEffect(() => {
    if (selectedNode) {
      setFormData({
        label: selectedNode.label,
        type: selectedNode.type,
        data: selectedNode.data,
      });
      setHasChanges(false);
    }
  }, [selectedNode]);

  if (!selectedNode) {
    return null;
  }

  const handleInputChange = (field: string, value: string | Record<string, unknown>) => {
    setFormData((prev) => ({
      ...prev,
      [field]: value,
    }));
    setHasChanges(true);
  };

  const handleDataChange = (field: string, value: string) => {
    setFormData((prev) => ({
      ...prev,
      data: {
        ...prev.data,
        [field]: value,
      },
    }));
    setHasChanges(true);
  };

  const handleSave = () => {
    trackEvent('properties.save', { nodeId: selectedNodeId || '' });

    if (selectedNodeId) {
      updateNode(selectedNodeId, formData);
      setHasChanges(false);
    }
  };

  const handleCancel = () => {
    trackEvent('properties.cancel', { nodeId: selectedNodeId || '' });

    if (selectedNode) {
      setFormData({
        label: selectedNode.label,
        type: selectedNode.type,
        data: selectedNode.data,
      });
      setHasChanges(false);
    }
  };

  const handleClose = () => {
    trackEvent('properties.close', { nodeId: selectedNodeId || '' });
    onClose();
  };

  return (
    <Card className="w-80 h-full border-l rounded-none shadow-lg">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <div>
          <CardTitle className="text-lg">Properties</CardTitle>
          <CardDescription className="text-xs mt-1">
            {selectedNode.type} node
          </CardDescription>
        </div>
        <Button variant="ghost" size="icon" onClick={handleClose}>
          <X className="h-4 w-4" />
        </Button>
      </CardHeader>

      <Separator />

      <CardContent className="pt-6 space-y-6 overflow-y-auto max-h-[calc(100vh-200px)]">
        {/* Node ID */}
        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">Node ID</Label>
          <div className="text-sm font-mono bg-muted p-2 rounded">{selectedNode.id}</div>
        </div>

        {/* Label */}
        <div className="space-y-2">
          <Label htmlFor="label">Label *</Label>
          <Input
            id="label"
            value={formData.label || ''}
            onChange={(e) => handleInputChange('label', e.target.value)}
            placeholder="Enter node label"
          />
        </div>

        {/* Type */}
        <div className="space-y-2">
          <Label htmlFor="type">Type *</Label>
          <Select
            value={formData.type || selectedNode.type}
            onValueChange={(value) => handleInputChange('type', value)}
          >
            <SelectTrigger id="type">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="task">Task</SelectItem>
              <SelectItem value="condition">Condition</SelectItem>
              <SelectItem value="start">Start</SelectItem>
              <SelectItem value="end">End</SelectItem>
              <SelectItem value="split">Split</SelectItem>
              <SelectItem value="join">Join</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Type-specific fields */}
        {formData.type === 'task' && (
          <>
            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={(formData.data?.description as string) || ''}
                onChange={(e) => handleDataChange('description', e.target.value)}
                placeholder="Enter task description"
                rows={3}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="documentation">Documentation</Label>
              <Textarea
                id="documentation"
                value={(formData.data?.documentation as string) || ''}
                onChange={(e) => handleDataChange('documentation', e.target.value)}
                placeholder="Enter task documentation"
                rows={4}
              />
            </div>
          </>
        )}

        {formData.type === 'condition' && (
          <>
            <div className="space-y-2">
              <Label htmlFor="condition">Condition Expression</Label>
              <Textarea
                id="condition"
                value={(formData.data?.condition as string) || ''}
                onChange={(e) => handleDataChange('condition', e.target.value)}
                placeholder="Enter XPath or XQuery expression"
                rows={3}
                className="font-mono text-xs"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={(formData.data?.description as string) || ''}
                onChange={(e) => handleDataChange('description', e.target.value)}
                placeholder="Enter condition description"
                rows={3}
              />
            </div>
          </>
        )}

        {/* Validation errors */}
        {nodeErrors.length > 0 && (
          <div className="space-y-2">
            <Label className="text-red-600">Validation Errors</Label>
            <div className="space-y-1">
              {nodeErrors.map((error, idx) => (
                <Badge key={idx} variant="destructive" className="w-full justify-start">
                  <span className="text-xs">{error.message}</span>
                </Badge>
              ))}
            </div>
          </div>
        )}

        {/* Position */}
        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">Position</Label>
          <div className="grid grid-cols-2 gap-2 text-sm font-mono">
            <div className="bg-muted p-2 rounded">
              <span className="text-muted-foreground">X:</span> {Math.round(selectedNode.position.x)}
            </div>
            <div className="bg-muted p-2 rounded">
              <span className="text-muted-foreground">Y:</span> {Math.round(selectedNode.position.y)}
            </div>
          </div>
        </div>

        {/* Action buttons */}
        <Separator />

        <div className="flex gap-2">
          <Button
            onClick={handleSave}
            disabled={!hasChanges}
            className="flex-1"
          >
            Save
          </Button>
          <Button
            variant="outline"
            onClick={handleCancel}
            disabled={!hasChanges}
            className="flex-1"
          >
            Cancel
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
