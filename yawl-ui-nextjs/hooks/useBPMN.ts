/**
 * useBPMN - Advanced BPMN diagram management hook
 * Handles BPMN 2.0 diagram visualization and editing
 * Integrates bpmn-js for full BPMN 2.0 support
 */

import { useEffect, useRef, useState } from 'react'
import BpmnModeler from 'bpmn-js/lib/Modeler'
import BpmnViewer from 'bpmn-js/lib/Viewer'

interface BPMNState {
  diagram: any | null
  modeler: BpmnModeler | null
  viewer: BpmnViewer | null
  xml: string | null
  isValid: boolean
  errors: string[]
  canUndo: boolean
  canRedo: boolean
}

/**
 * Hook for managing BPMN diagrams with bpmn-js
 */
export function useBPMN(containerRef: React.RefObject<HTMLDivElement>, editMode = true) {
  const [state, setState] = useState<BPMNState>({
    diagram: null,
    modeler: null,
    viewer: null,
    xml: null,
    isValid: true,
    errors: [],
    canUndo: false,
    canRedo: false,
  })

  const instanceRef = useRef<BpmnModeler | BpmnViewer | null>(null)

  /**
   * Initialize BPMN modeler or viewer
   */
  useEffect(() => {
    if (!containerRef.current) return

    const initBPMN = async () => {
      try {
        let instance: BpmnModeler | BpmnViewer

        if (editMode) {
          instance = new BpmnModeler({
            container: containerRef.current!,
            propertiesPanel: {
              parent: '#bpmn-properties',
            },
          })
        } else {
          instance = new BpmnViewer({
            container: containerRef.current!,
          })
        }

        instanceRef.current = instance

        // Handle changes
        if (editMode) {
          const modeler = instance as BpmnModeler
          modeler.on('commandStack.changed', () => {
            const commandStack = modeler.get('commandStack')
            setState((prev) => ({
              ...prev,
              canUndo: commandStack.canUndo(),
              canRedo: commandStack.canRedo(),
            }))
          })
        }

        setState((prev) => ({
          ...prev,
          [editMode ? 'modeler' : 'viewer']: instance,
        }))

        // Load default diagram
        const emptyDiagram = getDefaultBPMNDiagram()
        await instance.importXML(emptyDiagram)

        setState((prev) => ({
          ...prev,
          xml: emptyDiagram,
          diagram: await instance.getDefinitions(),
        }))
      } catch (err) {
        const error = err instanceof Error ? err.message : 'Failed to initialize BPMN'
        setState((prev) => ({
          ...prev,
          errors: [...prev.errors, error],
          isValid: false,
        }))
      }
    }

    initBPMN()

    return () => {
      if (instanceRef.current) {
        instanceRef.current.destroy()
      }
    }
  }, [editMode, containerRef])

  /**
   * Import BPMN XML
   */
  const importXML = async (xml: string) => {
    try {
      if (!instanceRef.current) return

      await instanceRef.current.importXML(xml)

      setState((prev) => ({
        ...prev,
        xml,
        isValid: true,
        errors: [],
      }))
    } catch (err) {
      const error = err instanceof Error ? err.message : 'Failed to import BPMN'
      setState((prev) => ({
        ...prev,
        isValid: false,
        errors: [...prev.errors, error],
      }))
    }
  }

  /**
   * Export BPMN XML
   */
  const exportXML = async (): Promise<string | null> => {
    try {
      if (!instanceRef.current) return null

      const { xml } = await instanceRef.current.saveXML({ format: true })
      setState((prev) => ({ ...prev, xml }))
      return xml
    } catch (err) {
      const error = err instanceof Error ? err.message : 'Failed to export BPMN'
      setState((prev) => ({
        ...prev,
        errors: [...prev.errors, error],
      }))
      return null
    }
  }

  /**
   * Export as SVG image
   */
  const exportSVG = async (): Promise<string | null> => {
    try {
      if (!instanceRef.current) return null

      const { svg } = await instanceRef.current.saveSVG({ format: true })
      return svg
    } catch (err) {
      const error = err instanceof Error ? err.message : 'Failed to export SVG'
      setState((prev) => ({
        ...prev,
        errors: [...prev.errors, error],
      }))
      return null
    }
  }

  /**
   * Undo last action (modeler only)
   */
  const undo = () => {
    if (!editMode || !instanceRef.current) return
    const modeler = instanceRef.current as BpmnModeler
    const commandStack = modeler.get('commandStack')
    commandStack.undo()
  }

  /**
   * Redo last undone action (modeler only)
   */
  const redo = () => {
    if (!editMode || !instanceRef.current) return
    const modeler = instanceRef.current as BpmnModeler
    const commandStack = modeler.get('commandStack')
    commandStack.redo()
  }

  /**
   * Get current diagram XML
   */
  const getCurrentXML = async (): Promise<string | null> => {
    return state.xml || exportXML()
  }

  /**
   * Zoom in
   */
  const zoomIn = () => {
    if (!instanceRef.current) return
    const canvas = instanceRef.current.get('canvas')
    canvas.zoom(1.2)
  }

  /**
   * Zoom out
   */
  const zoomOut = () => {
    if (!instanceRef.current) return
    const canvas = instanceRef.current.get('canvas')
    canvas.zoom(0.8)
  }

  /**
   * Fit to viewport
   */
  const fitToViewport = () => {
    if (!instanceRef.current) return
    const canvas = instanceRef.current.get('canvas')
    canvas.zoom('fit-viewport')
  }

  /**
   * Reset zoom to 100%
   */
  const resetZoom = () => {
    if (!instanceRef.current) return
    const canvas = instanceRef.current.get('canvas')
    canvas.zoom(1)
  }

  return {
    // State
    diagram: state.diagram,
    xml: state.xml,
    isValid: state.isValid,
    errors: state.errors,
    canUndo: state.canUndo,
    canRedo: state.canRedo,
    modeler: state.modeler,
    viewer: state.viewer,

    // Actions
    importXML,
    exportXML,
    exportSVG,
    undo,
    redo,
    getCurrentXML,
    zoomIn,
    zoomOut,
    fitToViewport,
    resetZoom,
  }
}

/**
 * Get default empty BPMN diagram
 */
function getDefaultBPMNDiagram(): string {
  return `<?xml version="1.0" encoding="UTF-8"?>
<bpmn2:definitions xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:bpmn2="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI" xmlns:dc="http://www.omg.org/spec/DD/20100524/DC" xmlns:di="http://www.omg.org/spec/DD/20100524/DI" xsi:schemaLocation="http://www.omg.org/spec/BPMN/20100524/MODEL BPMN20.xsd" id="sample-diagram" targetNamespace="http://bpmn.io/schema/bpmn" exporter="bpmn-js" exporterVersion="12.0.0">
  <bpmn2:process id="Process_1" isExecutable="false">
    <bpmn2:startEvent id="StartEvent_1" name="Start"/>
    <bpmn2:endEvent id="EndEvent_1" name="End"/>
    <bpmn2:sequenceFlow id="SequenceFlow_1" sourceRef="StartEvent_1" targetRef="EndEvent_1"/>
  </bpmn2:process>
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Process_1">
      <bpmndi:BPMNShape id="BPMNShape_StartEvent_1" bpmnElement="StartEvent_1">
        <dc:Bounds x="100" y="100" width="36" height="36"/>
      </bpmndi:BPMNShape>
      <bpmndi:BPMNShape id="BPMNShape_EndEvent_1" bpmnElement="EndEvent_1">
        <dc:Bounds x="400" y="100" width="36" height="36"/>
      </bpmndi:BPMNShape>
      <bpmndi:BPMNEdge id="BPMNEdge_SequenceFlow_1" bpmnElement="SequenceFlow_1">
        <di:waypoint x="136" y="118"/>
        <di:waypoint x="400" y="118"/>
      </bpmndi:BPMNEdge>
    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn2:definitions>`
}

export default useBPMN
