/**
 * BPMN Viewer Component
 * Display and interact with BPMN 2.0 diagrams
 * Built on bpmn-js Viewer
 */

'use client'

import React, { useRef } from 'react'
import { useBPMN } from '@/hooks/useBPMN'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import {
  ZoomIn,
  ZoomOut,
  Maximize,
  RotateCcw,
  Download,
  Upload,
} from 'lucide-react'
import 'bpmn-js/dist/assets/diagram-js.css'
import 'bpmn-js/dist/assets/bpmn-font/css/bpmn.css'

interface BPMNViewerProps {
  xml?: string
  onDiagramLoaded?: (xml: string) => void
}

/**
 * BPMN Diagram Viewer Component
 */
export function BPMNViewer({ xml, onDiagramLoaded }: BPMNViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const { exportXML, exportSVG, zoomIn, zoomOut, fitToViewport, resetZoom } =
    useBPMN(containerRef, false)

  const handleDownloadXML = async () => {
    const xmlContent = await exportXML()
    if (xmlContent) {
      downloadFile(xmlContent, 'diagram.bpmn', 'application/xml')
    }
  }

  const handleDownloadSVG = async () => {
    const svgContent = await exportSVG()
    if (svgContent) {
      downloadFile(svgContent, 'diagram.svg', 'image/svg+xml')
    }
  }

  return (
    <Card className="w-full h-full flex flex-col">
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">BPMN Diagram Viewer</CardTitle>
          <div className="flex gap-1">
            <Button size="sm" variant="outline" onClick={zoomIn} title="Zoom In">
              <ZoomIn className="h-4 w-4" />
            </Button>
            <Button size="sm" variant="outline" onClick={zoomOut} title="Zoom Out">
              <ZoomOut className="h-4 w-4" />
            </Button>
            <Button size="sm" variant="outline" onClick={fitToViewport} title="Fit">
              <Maximize className="h-4 w-4" />
            </Button>
            <Button size="sm" variant="outline" onClick={resetZoom} title="Reset">
              <RotateCcw className="h-4 w-4" />
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={handleDownloadXML}
              title="Download XML"
            >
              <Download className="h-4 w-4" />
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={handleDownloadSVG}
              title="Download SVG"
            >
              <Download className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="flex-1 p-0 overflow-hidden">
        <div
          ref={containerRef}
          className="w-full h-full"
          style={{ minHeight: '600px' }}
        />
      </CardContent>
    </Card>
  )
}

/**
 * BPMN Modeler Component
 * Edit BPMN 2.0 diagrams
 */
export function BPMNModeler({ xml, onDiagramChanged }: BPMNViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const {
    exportXML,
    exportSVG,
    zoomIn,
    zoomOut,
    fitToViewport,
    resetZoom,
    undo,
    redo,
    canUndo,
    canRedo,
  } = useBPMN(containerRef, true)

  const handleDownloadXML = async () => {
    const xmlContent = await exportXML()
    if (xmlContent) {
      downloadFile(xmlContent, 'diagram.bpmn', 'application/xml')
      onDiagramChanged?.(xmlContent)
    }
  }

  const handleDownloadSVG = async () => {
    const svgContent = await exportSVG()
    if (svgContent) {
      downloadFile(svgContent, 'diagram.svg', 'image/svg+xml')
    }
  }

  return (
    <div className="space-y-4">
      <Card className="w-full">
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg">BPMN Diagram Editor</CardTitle>
            <div className="flex gap-1">
              <Button
                size="sm"
                variant="outline"
                onClick={undo}
                disabled={!canUndo}
                title="Undo"
              >
                ↶
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={redo}
                disabled={!canRedo}
                title="Redo"
              >
                ↷
              </Button>
              <Button size="sm" variant="outline" onClick={zoomIn} title="Zoom In">
                <ZoomIn className="h-4 w-4" />
              </Button>
              <Button size="sm" variant="outline" onClick={zoomOut} title="Zoom Out">
                <ZoomOut className="h-4 w-4" />
              </Button>
              <Button size="sm" variant="outline" onClick={fitToViewport} title="Fit">
                <Maximize className="h-4 w-4" />
              </Button>
              <Button size="sm" variant="outline" onClick={resetZoom} title="Reset">
                <RotateCcw className="h-4 w-4" />
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={handleDownloadXML}
                title="Download XML"
              >
                <Download className="h-4 w-4" />
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={handleDownloadSVG}
                title="Download SVG"
              >
                <Download className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </CardHeader>

        <CardContent className="p-0">
          <div
            ref={containerRef}
            className="w-full h-full"
            style={{ minHeight: '600px' }}
          />
        </CardContent>
      </Card>

      <div id="bpmn-properties" className="bg-background border rounded-lg p-4">
        <p className="text-sm text-muted-foreground">
          Properties panel will appear here when you select elements
        </p>
      </div>
    </div>
  )
}

/**
 * Utility to download file
 */
function downloadFile(content: string, filename: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
  URL.revokeObjectURL(url)
}

export default BPMNViewer
