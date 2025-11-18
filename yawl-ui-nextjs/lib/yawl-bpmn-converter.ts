/**
 * YAWL to BPMN Conversion Service
 * Bidirectional conversion between YAWL and BPMN 2.0
 */

import type { YAWLSpecification, YAWLTask, ControlFlow } from '@/types/yawl'

export interface BPMNConversionOptions {
  includeMetadata?: boolean
  preserveIds?: boolean
  autoLayout?: boolean
}

/**
 * Service for converting between YAWL and BPMN formats
 */
export class YAWLBPMNConverterService {
  /**
   * Convert YAWL specification to BPMN XML
   */
  static yawlToBPMN(
    yawlSpec: YAWLSpecification,
    options: BPMNConversionOptions = {}
  ): string {
    const processId = `Process_${yawlSpec.id}`
    const taskElements = this.createBPMNTasks(yawlSpec.tasks)
    const flowElements = this.createBPMNFlows(
      yawlSpec.nets[0]?.flows || [],
      yawlSpec.tasks
    )
    const shapes = this.createBPMNShapes(yawlSpec.tasks)
    const edges = this.createBPMNEdges(yawlSpec.nets[0]?.flows || [])

    return `<?xml version="1.0" encoding="UTF-8"?>
<bpmn2:definitions xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:bpmn2="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI" xmlns:dc="http://www.omg.org/spec/DD/20100524/DC" xmlns:di="http://www.omg.org/spec/DD/20100524/DI" id="definitions" targetNamespace="http://bpmn.io/schema/bpmn" exporter="YAWL-BPMN-Converter" exporterVersion="1.0.0">
  <bpmn2:process id="${processId}" isExecutable="false" name="${yawlSpec.name}">
    <bpmn2:startEvent id="StartEvent_1" name="Start"/>
    ${taskElements}
    <bpmn2:endEvent id="EndEvent_1" name="End"/>
    ${flowElements}
  </bpmn2:process>
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="${processId}">
      <bpmndi:BPMNShape id="BPMNShape_StartEvent_1" bpmnElement="StartEvent_1">
        <dc:Bounds x="100" y="100" width="36" height="36"/>
      </bpmndi:BPMNShape>
      ${shapes}
      <bpmndi:BPMNShape id="BPMNShape_EndEvent_1" bpmnElement="EndEvent_1">
        <dc:Bounds x="900" y="100" width="36" height="36"/>
      </bpmndi:BPMNShape>
      ${edges}
    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn2:definitions>`
  }

  /**
   * Convert BPMN XML to YAWL specification
   */
  static bpmnToYAWL(bpmnXml: string): YAWLSpecification {
    // Parse BPMN XML
    const parser = new DOMParser()
    const doc = parser.parseFromString(bpmnXml, 'application/xml')

    // Extract process info
    const processEl = doc.querySelector('bpmn2\\:process')
    const processId = processEl?.getAttribute('id') || `wf-${Date.now()}`
    const processName = processEl?.getAttribute('name') || 'Converted Workflow'

    // Extract tasks
    const tasks = this.extractBPMNTasks(doc)

    // Extract flows
    const flows = this.extractBPMNFlows(doc)

    return {
      id: processId,
      uri: `http://example.com/workflow/${processId}`,
      name: processName,
      version: '1.0',
      xmlns: 'http://www.yawlfoundation.org/yawl/',
      isValid: true,
      tasks,
      nets: [
        {
          id: `net_${processId}`,
          name: `Network for ${processName}`,
          tasks,
          flows,
        },
      ],
      metadata: {
        title: processName,
        dateModified: new Date(),
      },
    }
  }

  /**
   * Create BPMN task elements from YAWL tasks
   */
  private static createBPMNTasks(tasks: YAWLTask[]): string {
    return tasks
      .map((task) => {
        const taskType = this.mapYAWLTaskTypeToBPMN(task.type)
        return `<bpmn2:${taskType} id="Task_${task.id}" name="${task.name}"/>`
      })
      .join('\n    ')
  }

  /**
   * Create BPMN sequence flows from control flows
   */
  private static createBPMNFlows(
    flows: ControlFlow[],
    tasks: YAWLTask[]
  ): string {
    let result = `<bpmn2:sequenceFlow id="SequenceFlow_Start" sourceRef="StartEvent_1" targetRef="Task_${tasks[0]?.id || 'first'}"/>\n    `

    result += flows
      .map((flow, idx) => {
        return `<bpmn2:sequenceFlow id="SequenceFlow_${idx}" sourceRef="Task_${flow.source}" targetRef="Task_${flow.target}"/>`
      })
      .join('\n    ')

    result += `\n    <bpmn2:sequenceFlow id="SequenceFlow_End" sourceRef="Task_${tasks[tasks.length - 1]?.id || 'last'}" targetRef="EndEvent_1"/>`

    return result
  }

  /**
   * Create BPMN shapes (visual positions)
   */
  private static createBPMNShapes(tasks: YAWLTask[]): string {
    return tasks
      .map((task, idx) => {
        const x = 200 + idx * 150
        const y = 100
        return `<bpmndi:BPMNShape id="BPMNShape_Task_${task.id}" bpmnElement="Task_${task.id}">
        <dc:Bounds x="${x}" y="${y}" width="100" height="80"/>
      </bpmndi:BPMNShape>`
      })
      .join('\n      ')
  }

  /**
   * Create BPMN edges (visual connections)
   */
  private static createBPMNEdges(flows: ControlFlow[]): string {
    return flows
      .map((flow, idx) => {
        return `<bpmndi:BPMNEdge id="BPMNEdge_SequenceFlow_${idx}" bpmnElement="SequenceFlow_${idx}">
        <di:waypoint x="250" y="140"/>
        <di:waypoint x="350" y="140"/>
      </bpmndi:BPMNEdge>`
      })
      .join('\n      ')
  }

  /**
   * Extract tasks from BPMN XML
   */
  private static extractBPMNTasks(doc: Document): YAWLTask[] {
    const tasks: YAWLTask[] = []
    const taskElements = doc.querySelectorAll('bpmn2\\:task, bpmn2\\:serviceTask, bpmn2\\:userTask')

    taskElements.forEach((el) => {
      const id = el.getAttribute('id') || `task_${Date.now()}`
      const name = el.getAttribute('name') || id

      tasks.push({
        id,
        name,
        type: 'atomic',
        documentation: `Converted from BPMN: ${name}`,
      })
    })

    return tasks
  }

  /**
   * Extract flows from BPMN XML
   */
  private static extractBPMNFlows(doc: Document): ControlFlow[] {
    const flows: ControlFlow[] = []
    const flowElements = doc.querySelectorAll('bpmn2\\:sequenceFlow')

    flowElements.forEach((el) => {
      const id = el.getAttribute('id') || `flow_${Date.now()}`
      const source = el.getAttribute('sourceRef') || ''
      const target = el.getAttribute('targetRef') || ''

      if (source && target) {
        flows.push({
          id,
          source: source.replace('Task_', ''),
          target: target.replace('Task_', ''),
          pattern: 'sequence',
        })
      }
    })

    return flows
  }

  /**
   * Map YAWL task type to BPMN element type
   */
  private static mapYAWLTaskTypeToBPMN(yawlType: string): string {
    switch (yawlType) {
      case 'composite':
        return 'subProcess'
      case 'multi-instance':
      case 'multi-instance-dynamic':
        return 'serviceTask'
      default:
        return 'task'
    }
  }

  /**
   * Validate BPMN XML structure
   */
  static validateBPMN(bpmnXml: string): { valid: boolean; errors: string[] } {
    const errors: string[] = []

    try {
      const parser = new DOMParser()
      const doc = parser.parseFromString(bpmnXml, 'application/xml')

      // Check for parse errors
      if (doc.getElementsByTagName('parsererror').length > 0) {
        errors.push('Invalid XML syntax')
        return { valid: false, errors }
      }

      // Check for required BPMN elements
      if (!doc.querySelector('bpmn2\\:definitions')) {
        errors.push('Missing bpmn2:definitions element')
      }

      if (!doc.querySelector('bpmn2\\:process')) {
        errors.push('Missing bpmn2:process element')
      }

      return {
        valid: errors.length === 0,
        errors,
      }
    } catch (err) {
      return {
        valid: false,
        errors: [err instanceof Error ? err.message : 'Unknown error'],
      }
    }
  }

  /**
   * Get BPMN diagram statistics
   */
  static getBPMNStats(bpmnXml: string): {
    taskCount: number
    flowCount: number
    eventCount: number
    gateways: number
  } {
    const parser = new DOMParser()
    const doc = parser.parseFromString(bpmnXml, 'application/xml')

    return {
      taskCount: doc.querySelectorAll('bpmn2\\:task').length,
      flowCount: doc.querySelectorAll('bpmn2\\:sequenceFlow').length,
      eventCount: doc.querySelectorAll('[id*="Event"]').length,
      gateways: doc.querySelectorAll('bpmn2\\:gateway').length,
    }
  }
}

export default YAWLBPMNConverterService
