# KNHK WebAssembly Bindings

[![Crates.io](https://img.shields.io/crates/v/knhk-wasm.svg)](https://crates.io/crates/knhk-wasm)
[![Documentation](https://docs.rs/knhk-wasm/badge.svg)](https://docs.rs/knhk-wasm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

WebAssembly bindings for the KNHK workflow engine, enabling portable workflow execution across browsers, Node.js, Deno, and edge computing platforms.

## Features

- **🌍 Universal**: Run workflows anywhere WebAssembly is supported
- **⚡ Fast**: Near-native performance (~1.5-2x overhead)
- **📦 Compact**: < 500KB compressed (gzip/brotli)
- **🔒 Secure**: Sandboxed execution with resource limits
- **📘 Type-Safe**: Full TypeScript definitions included
- **🎯 Zero-Copy**: Efficient JavaScript interop

## Quick Start

### Browser

```html
<script type="module">
  import init, { WasmWorkflowEngine } from './knhk_wasm.js';

  await init();
  const engine = new WasmWorkflowEngine();

  const workflow = {
    id: "user-registration",
    pattern: "Sequence",
    tasks: [
      { id: "validate", type: "validate" },
      { id: "create", type: "transform" }
    ]
  };

  const result = await engine.execute_workflow_json(
    JSON.stringify(workflow),
    { email: "user@example.com" }
  );

  console.log('Result:', result);
</script>
```

### Node.js

```bash
npm install @knhk/wasm
```

```javascript
import { WasmWorkflowEngine } from '@knhk/wasm';

const engine = new WasmWorkflowEngine();
const result = await engine.execute_workflow_json(
  JSON.stringify(workflow),
  inputData
);
```

### TypeScript (High-level API)

```typescript
import { WorkflowEngine, WorkflowDefinition } from '@knhk/wasm';

const engine = await WorkflowEngine.create({
  maxWorkflows: 100,
  enableTelemetry: true,
  timeoutMs: 30000
});

const workflow: WorkflowDefinition = {
  id: "order-processing",
  pattern: "Sequence",
  tasks: [/*...*/]
};

const result = await engine.execute(workflow, { orderId: "123" });
```

## Building from Source

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack
```

### Build

```bash
# Build for web
wasm-pack build --target web --release

# Build for Node.js
wasm-pack build --target nodejs --release

# Build for bundlers (webpack/vite/rollup)
wasm-pack build --target bundler --release
```

### Optimize

```bash
# Install optimizer
cargo install wasm-opt

# Optimize binary
wasm-opt -Oz --enable-simd -o optimized.wasm knhk_wasm_bg.wasm
```

## Workflow Patterns

### Sequence

Execute tasks in order:

```javascript
{
  id: "sequential-flow",
  pattern: "Sequence",
  tasks: [
    { id: "step1", type: "validate" },
    { id: "step2", type: "transform" },
    { id: "step3", type: "compute" }
  ]
}
```

### Parallel

Execute tasks concurrently:

```javascript
{
  id: "parallel-flow",
  pattern: "Parallel",
  tasks: [
    { id: "check-email", type: "validate" },
    { id: "check-phone", type: "validate" },
    { id: "check-address", type: "validate" }
  ]
}
```

### Choice

Conditional branching:

```javascript
{
  id: "choice-flow",
  pattern: "Choice",
  tasks: [
    {
      id: "express-path",
      type: "transform",
      condition: "express",
      config: { shippingDays: 1 }
    },
    {
      id: "standard-path",
      type: "transform",
      condition: "standard",
      config: { shippingDays: 5 }
    }
  ]
}
```

### Loop

Iterative execution:

```javascript
{
  id: "loop-flow",
  pattern: "Loop",
  loopCondition: "continue",
  tasks: [
    { id: "process", type: "compute" },
    { id: "check", type: "validate" }
  ]
}
```

## API Reference

### WasmWorkflowEngine

```typescript
class WasmWorkflowEngine {
  constructor(): WasmWorkflowEngine;

  with_config(config: WasmEngineConfig): WasmWorkflowEngine;

  execute_workflow(
    workflowDef: string,
    inputData: any
  ): Promise<any>;

  execute_workflow_json(
    workflowJson: string,
    inputData: any
  ): Promise<any>;

  validate_workflow(workflowDef: string): boolean;

  get_stats(): EngineStats;

  reset(): void;

  static version(): string;
}
```

### WasmEngineConfig

```typescript
class WasmEngineConfig {
  max_workflows: number;          // Default: 100
  enable_telemetry: boolean;      // Default: true
  timeout_ms: number;             // Default: 30000
}
```

### WorkflowEngine (High-level API)

```typescript
class WorkflowEngine {
  static create(config?: EngineConfig): Promise<WorkflowEngine>;

  execute(
    workflow: WorkflowDefinition | string,
    input?: any
  ): Promise<any>;

  validate(workflow: WorkflowDefinition | string): boolean;

  registerHostFunction(name: string, func: HostFunction): void;

  getStats(): EngineStats;

  reset(): void;
}
```

## Performance

### Benchmarks

| Operation | Native | WASM | Overhead |
|-----------|--------|------|----------|
| Simple workflow | 0.3ms | 0.5ms | 1.67x |
| Complex workflow | 3ms | 5ms | 1.67x |
| 100 workflows | 30ms | 50ms | 1.67x |

**Throughput**: ~2,000 workflows/second (WASM) vs ~3,300 (native)

### Binary Size

- **Uncompressed**: 1.2 MB
- **Gzipped**: 450 KB
- **Brotli**: 380 KB

## Examples

### Browser Example

See [examples/wasm-browser/](../../examples/wasm-browser/) for a complete browser example with:
- Interactive workflow editor
- Real-time execution
- Statistics dashboard
- Error handling

### Node.js Example

See [examples/wasm-nodejs/](../../examples/wasm-nodejs/) for server-side usage:
- Multiple workflow patterns
- Performance benchmarking
- Error handling
- Statistics tracking

### React Integration

```typescript
import { useEffect, useState } from 'react';
import { WorkflowEngine } from '@knhk/wasm';

function useWorkflow() {
  const [engine, setEngine] = useState<WorkflowEngine | null>(null);

  useEffect(() => {
    WorkflowEngine.create().then(setEngine);
  }, []);

  const execute = async (workflow: any, input: any) => {
    if (!engine) throw new Error('Engine not initialized');
    return await engine.execute(workflow, input);
  };

  return { execute, ready: !!engine };
}
```

## Deployment

### Web Server (Static Files)

```nginx
# nginx.conf
location ~* \.wasm$ {
    add_header Content-Type application/wasm;
    add_header Cache-Control "public, max-age=31536000, immutable";
    gzip on;
}
```

### Cloudflare Workers

```javascript
import { WasmWorkflowEngine } from './knhk_wasm.js';

export default {
  async fetch(request) {
    const engine = new WasmWorkflowEngine();
    const { workflow, input } = await request.json();
    const result = await engine.execute_workflow_json(
      JSON.stringify(workflow),
      input
    );
    return new Response(JSON.stringify(result));
  }
};
```

### AWS Lambda

```javascript
import { WasmWorkflowEngine } from '@knhk/wasm';

const engine = new WasmWorkflowEngine();

export const handler = async (event) => {
  const { workflow, input } = JSON.parse(event.body);
  const result = await engine.execute_workflow_json(
    JSON.stringify(workflow),
    input
  );
  return { statusCode: 200, body: JSON.stringify(result) };
};
```

## Troubleshooting

### WASM Module Not Found

Ensure the `.wasm` file is served with correct MIME type:

```javascript
// Vite config
export default {
  server: {
    headers: {
      'Content-Type': 'application/wasm'
    }
  }
};
```

### Memory Issues

Increase WASM memory limit:

```javascript
const memory = new WebAssembly.Memory({
  initial: 256,  // 16MB
  maximum: 512   // 32MB
});
```

### Large Bundle Size

Use dynamic imports:

```javascript
const { WasmWorkflowEngine } = await import('@knhk/wasm');
```

## Documentation

- [Deployment Guide](../../docs/wasm-deployment.md)
- [API Documentation](https://docs.rs/knhk-wasm)
- [KNHK Main Docs](https://docs.knhk.io)

## License

MIT License - see [LICENSE](../../LICENSE) for details

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md)

## Support

- GitHub Issues: https://github.com/yourusername/knhk/issues
- Discord: https://discord.gg/knhk
- Email: support@knhk.io
