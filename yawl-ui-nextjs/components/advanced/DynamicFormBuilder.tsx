/**
 * Dynamic Form Builder Component
 * Creates forms from YAWL task definitions
 * Supports pattern constraints and validation
 */

'use client'

import React, { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import type { YAWLTask, YAWLVariable } from '@/types/yawl'

interface DynamicFormBuilderProps {
  task: YAWLTask
  onSubmit?: (data: Record<string, unknown>) => void
  onChange?: (data: Record<string, unknown>) => void
}

/**
 * Dynamically builds forms from YAWL task definitions
 * Handles various field types and validation
 */
export function DynamicFormBuilder({
  task,
  onSubmit,
  onChange,
}: DynamicFormBuilderProps) {
  const [formData, setFormData] = useState<Record<string, unknown>>({})
  const [errors, setErrors] = useState<Record<string, string>>({})

  const variables = task.inputData?.variables || []

  const handleChange = (
    name: string,
    value: unknown,
    variable: YAWLVariable
  ) => {
    const newData = { ...formData, [name]: value }
    setFormData(newData)

    // Validate if needed
    if (variable.type === 'number') {
      const numValue = Number(value)
      if (isNaN(numValue)) {
        setErrors((prev) => ({ ...prev, [name]: 'Must be a number' }))
      } else {
        const newErrors = { ...errors }
        delete newErrors[name]
        setErrors(newErrors)
      }
    }

    onChange?.(newData)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()

    if (Object.keys(errors).length === 0) {
      onSubmit?.(formData)
    }
  }

  const renderField = (variable: YAWLVariable) => {
    const value = formData[variable.name] || ''
    const error = errors[variable.name]

    switch (variable.type) {
      case 'string':
        return (
          <input
            type="text"
            value={String(value)}
            onChange={(e) =>
              handleChange(variable.name, e.target.value, variable)
            }
            className="w-full px-3 py-2 border border-input rounded-md bg-background"
            placeholder={variable.documentation}
          />
        )
      case 'number':
        return (
          <input
            type="number"
            value={String(value)}
            onChange={(e) =>
              handleChange(variable.name, e.target.value, variable)
            }
            className="w-full px-3 py-2 border border-input rounded-md bg-background"
            placeholder={variable.documentation}
          />
        )
      case 'boolean':
        return (
          <input
            type="checkbox"
            checked={Boolean(value)}
            onChange={(e) =>
              handleChange(variable.name, e.target.checked, variable)
            }
            className="w-4 h-4"
          />
        )
      case 'date':
        return (
          <input
            type="date"
            value={String(value)}
            onChange={(e) =>
              handleChange(variable.name, e.target.value, variable)
            }
            className="w-full px-3 py-2 border border-input rounded-md bg-background"
          />
        )
      default:
        return (
          <textarea
            value={String(value)}
            onChange={(e) =>
              handleChange(variable.name, e.target.value, variable)
            }
            className="w-full px-3 py-2 border border-input rounded-md bg-background"
            placeholder={variable.documentation}
            rows={3}
          />
        )
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">
          Task: {task.name}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {variables.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No input fields for this task
            </p>
          ) : (
            variables.map((variable) => (
              <div key={variable.name} className="space-y-1">
                <label className="block text-sm font-medium">
                  {variable.name}
                  {variable.documentation && (
                    <span className="text-xs text-muted-foreground ml-1">
                      ({variable.documentation})
                    </span>
                  )}
                </label>
                {renderField(variable)}
                {errors[variable.name] && (
                  <p className="text-xs text-red-600">
                    {errors[variable.name]}
                  </p>
                )}
              </div>
            ))
          )}

          <div className="flex gap-2 pt-4">
            <Button type="submit">Submit</Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setFormData({})
                setErrors({})
              }}
            >
              Clear
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}

export default DynamicFormBuilder
