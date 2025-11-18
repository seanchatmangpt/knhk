/**
 * BPMN Pages - Viewer and Editor
 */

'use client'

import { useRef, useState } from 'react'
import { BPMNModeler, BPMNViewer } from '@/components/bpmn/BPMNViewer'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Upload, ArrowLeftRight } from 'lucide-react'
import YAWLBPMNConverterService from '@/lib/yawl-bpmn-converter'

export default function BPMNPage() {
  const [activeTab, setActiveTab] = useState('editor')
  const [bpmnXml, setBpmnXml] = useState<string>('')
  const [conversionResult, setConversionResult] = useState<any>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    const text = await file.text()
    setBpmnXml(text)

    // Validate BPMN
    const validation = YAWLBPMNConverterService.validateBPMN(text)
    if (validation.valid) {
      const stats = YAWLBPMNConverterService.getBPMNStats(text)
      console.log('BPMN Statistics:', stats)
    }
  }

  const handleConvertToYAWL = () => {
    if (!bpmnXml) {
      alert('Please load a BPMN diagram first')
      return
    }

    try {
      const yawlSpec = YAWLBPMNConverterService.bpmnToYAWL(bpmnXml)
      setConversionResult({
        type: 'yawl',
        data: yawlSpec,
      })
      alert('Converted to YAWL successfully!')
    } catch (err) {
      alert('Conversion failed: ' + (err instanceof Error ? err.message : 'Unknown error'))
    }
  }

  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">BPMN Diagram Editor</h1>
        <p className="text-muted-foreground mt-2">
          View, edit, and convert BPMN 2.0 diagrams using bpmn-js
        </p>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList>
          <TabsTrigger value="editor">Editor</TabsTrigger>
          <TabsTrigger value="viewer">Viewer</TabsTrigger>
          <TabsTrigger value="convert">Convert</TabsTrigger>
        </TabsList>

        {/* Editor Tab */}
        <TabsContent value="editor" className="mt-4">
          <BPMNModeler
            onDiagramChanged={(xml) => {
              setBpmnXml(xml)
            }}
          />
        </TabsContent>

        {/* Viewer Tab */}
        <TabsContent value="viewer" className="mt-4">
          <div className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Load BPMN File</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex gap-2">
                  <input
                    ref={fileInputRef}
                    type="file"
                    accept=".bpmn,.xml"
                    onChange={handleFileUpload}
                    className="hidden"
                  />
                  <Button onClick={() => fileInputRef.current?.click()}>
                    <Upload className="h-4 w-4 mr-2" />
                    Upload BPMN File
                  </Button>
                </div>
              </CardContent>
            </Card>

            {bpmnXml && <BPMNViewer xml={bpmnXml} />}
          </div>
        </TabsContent>

        {/* Convert Tab */}
        <TabsContent value="convert" className="mt-4">
          <div className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">YAWL ↔ BPMN Conversion</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <Button onClick={handleConvertToYAWL} className="w-full">
                  <ArrowLeftRight className="h-4 w-4 mr-2" />
                  Convert to YAWL
                </Button>

                {conversionResult && (
                  <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                    <p className="font-semibold text-green-900 mb-2">
                      Conversion Result:
                    </p>
                    <pre className="text-xs overflow-auto max-h-96 text-green-800">
                      {JSON.stringify(conversionResult.data, null, 2)}
                    </pre>
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}
