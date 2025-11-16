# KNHK WASM Quick Start Guide

Get started with KNHK WebAssembly in 5 minutes.

## Installation

### Option 1: NPM Package (Recommended)

```bash
npm install @knhk/wasm
```

### Option 2: CDN (For quick prototyping)

```html
<script type="module">
  import init, { WasmWorkflowEngine } from 'https://unpkg.com/@knhk/wasm';
  // Use engine...
</script>
```

### Option 3: Build from Source

```bash
git clone https://github.com/yourusername/knhk
cd knhk/rust/knhk-wasm
make build
```

## Hello World

### Browser

Create `index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <title>KNHK WASM Hello World</title>
</head>
<body>
    <h1>KNHK WASM Example</h1>
    <button id="runBtn">Run Workflow</button>
    <pre id="output"></pre>

    <script type="module">
        import init, { WasmWorkflowEngine } from '@knhk/wasm';

        // Initialize WASM
        await init();
        const engine = new WasmWorkflowEngine();

        // Define workflow
        const workflow = {
            id: "hello-world",
            pattern: "Sequence",
            tasks: [
                {
                    id: "greet",
                    type: "transform",
                    config: { greeting: "Hello, WASM!" }
                }
            ]
        };

        // Run on button click
        document.getElementById('runBtn').onclick = async () => {
            const result = await engine.execute_workflow_json(
                JSON.stringify(workflow),
                { user: "World" }
            );

            document.getElementById('output').textContent =
                JSON.stringify(result, null, 2);
        };
    </script>
</body>
</html>
```

Serve with any static server:

```bash
python -m http.server 8000
# Open http://localhost:8000
```

### Node.js

Create `app.js`:

```javascript
import { WasmWorkflowEngine } from '@knhk/wasm';

// Create engine
const engine = new WasmWorkflowEngine();

// Define workflow
const workflow = {
    id: "hello-world",
    pattern: "Sequence",
    tasks: [
        {
            id: "greet",
            type: "transform",
            config: { greeting: "Hello, WASM!" }
        }
    ]
};

// Execute
const result = await engine.execute_workflow_json(
    JSON.stringify(workflow),
    { user: "World" }
);

console.log('Result:', result);
```

Run:

```bash
node app.js
```

## Common Patterns

### 1. Sequential Processing

```javascript
const workflow = {
    id: "user-registration",
    pattern: "Sequence",
    tasks: [
        { id: "validate", type: "validate" },
        { id: "create", type: "transform" },
        { id: "notify", type: "compute" }
    ]
};
```

### 2. Parallel Execution

```javascript
const workflow = {
    id: "multi-validation",
    pattern: "Parallel",
    tasks: [
        { id: "check-email", type: "validate" },
        { id: "check-phone", type: "validate" },
        { id: "check-address", type: "validate" }
    ]
};
```

### 3. Conditional Branching

```javascript
const workflow = {
    id: "shipping-choice",
    pattern: "Choice",
    tasks: [
        {
            id: "express",
            type: "transform",
            condition: "express",
            config: { days: 1 }
        },
        {
            id: "standard",
            type: "transform",
            condition: "standard",
            config: { days: 5 }
        }
    ]
};
```

### 4. Loops

```javascript
const workflow = {
    id: "batch-processing",
    pattern: "Loop",
    loopCondition: "hasMore",
    tasks: [
        { id: "process-item", type: "compute" },
        { id: "check-next", type: "validate" }
    ]
};
```

## Configuration

### Engine Configuration

```javascript
import { WasmEngineConfig, WasmWorkflowEngine } from '@knhk/wasm';

const config = new WasmEngineConfig();
config.set_max_workflows(50);
config.set_enable_telemetry(true);
config.set_timeout_ms(10000);

const engine = WasmWorkflowEngine.with_config(config);
```

### High-Level API

```javascript
import { WorkflowEngine } from '@knhk/wasm';

const engine = await WorkflowEngine.create({
    maxWorkflows: 100,
    enableTelemetry: true,
    timeoutMs: 30000
});
```

## Error Handling

```javascript
try {
    const result = await engine.execute_workflow_json(
        JSON.stringify(workflow),
        input
    );
    console.log('Success:', result);
} catch (error) {
    console.error('Workflow failed:', error.message);
    // Handle error...
}
```

## Validation

```javascript
// Validate before executing
try {
    const isValid = engine.validate_workflow(
        JSON.stringify(workflow)
    );
    console.log('Workflow is valid!');
} catch (error) {
    console.error('Validation failed:', error.message);
}
```

## Statistics

```javascript
// Get engine stats
const stats = engine.get_stats();
console.log('Total executed:', stats.totalExecuted);
console.log('Running:', stats.runningWorkflows);
console.log('Failed:', stats.failedWorkflows);
console.log('Avg time:', stats.avgExecutionTimeMs, 'ms');
console.log('Memory:', stats.memoryUsageBytes / 1024, 'KB');
```

## TypeScript Support

```typescript
import {
    WorkflowEngine,
    WorkflowDefinition,
    EngineConfig
} from '@knhk/wasm';

const config: EngineConfig = {
    maxWorkflows: 100,
    enableTelemetry: true,
    timeoutMs: 30000
};

const engine = await WorkflowEngine.create(config);

const workflow: WorkflowDefinition = {
    id: "typed-workflow",
    pattern: "Sequence",
    tasks: [
        {
            id: "step1",
            type: "validate",
            config: { required: ["email"] }
        }
    ]
};

const result = await engine.execute(workflow, { email: "test@example.com" });
```

## React Integration

```typescript
import { useEffect, useState } from 'react';
import { WorkflowEngine } from '@knhk/wasm';

function App() {
    const [engine, setEngine] = useState<WorkflowEngine | null>(null);
    const [result, setResult] = useState<any>(null);

    useEffect(() => {
        WorkflowEngine.create().then(setEngine);
    }, []);

    const runWorkflow = async () => {
        if (!engine) return;

        const workflow = {
            id: "react-workflow",
            pattern: "Sequence",
            tasks: [{ id: "task1", type: "compute" }]
        };

        const output = await engine.execute(workflow, {});
        setResult(output);
    };

    if (!engine) return <div>Loading WASM...</div>;

    return (
        <div>
            <button onClick={runWorkflow}>Run Workflow</button>
            {result && <pre>{JSON.stringify(result, null, 2)}</pre>}
        </div>
    );
}
```

## Next Steps

- [Full API Reference](../rust/knhk-wasm/README.md)
- [Deployment Guide](./wasm-deployment.md)
- [Browser Example](../examples/wasm-browser/)
- [Node.js Example](../examples/wasm-nodejs/)
- [Performance Tuning](./wasm-deployment.md#performance-optimization)

## Common Issues

### WASM Module Not Loading

**Problem**: Module fails to load

**Solution**:
```javascript
// Check WASM support
if (!WebAssembly) {
    alert('WebAssembly not supported');
}

// Use absolute path
import init from '/path/to/knhk_wasm.js';
```

### Large Bundle Size

**Problem**: Bundle too large for production

**Solution**:
```javascript
// Use dynamic imports
const loadWasm = async () => {
    const { WasmWorkflowEngine } = await import('@knhk/wasm');
    return new WasmWorkflowEngine();
};
```

### CORS Errors

**Problem**: CORS blocking WASM file

**Solution**:
```nginx
# nginx.conf
add_header Access-Control-Allow-Origin *;
add_header Access-Control-Allow-Methods "GET, OPTIONS";
```

## Support

- Documentation: https://docs.knhk.io
- GitHub: https://github.com/yourusername/knhk
- Discord: https://discord.gg/knhk
- Email: support@knhk.io
