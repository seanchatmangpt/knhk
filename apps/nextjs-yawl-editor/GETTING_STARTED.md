# Getting Started with YAWL Process Editor

This guide will help you get up and running with the YAWL Process Editor.

## Quick Start

### 1. Install Dependencies

```bash
cd /home/user/knhk/apps/nextjs-yawl-editor
npm install
```

### 2. Start Development Server

```bash
npm run dev
```

The application will be available at [http://localhost:3000](http://localhost:3000).

### 3. Open the Editor

Navigate to [http://localhost:3000/editor](http://localhost:3000/editor) to start creating workflows.

## Development Workflow

### File Structure Overview

```
src/
├── app/                    # Routes (Next.js App Router)
│   ├── layout.tsx         # Root layout
│   ├── page.tsx           # Home page
│   └── editor/
│       └── page.tsx       # Editor page
├── components/            # React components
├── lib/                   # Core utilities
│   ├── rdf/              # RDF/Turtle operations
│   ├── validation/       # Pattern validation
│   ├── telemetry/        # OpenTelemetry setup
│   └── types/            # TypeScript types
├── store/                # Zustand stores
└── styles/               # Global CSS
```

### Adding a New Component

1. Create component file in `src/components/`:

```tsx
// src/components/editor/PatternPalette.tsx
import { FC } from 'react';

interface PatternPaletteProps {
  onSelectPattern: (pattern: string) => void;
}

export const PatternPalette: FC<PatternPaletteProps> = ({ onSelectPattern }) => {
  return (
    <div className="pattern-palette">
      {/* Component implementation */}
    </div>
  );
};
```

2. Use the component:

```tsx
import { PatternPalette } from '@/components/editor/PatternPalette';

// In your page/component
<PatternPalette onSelectPattern={handlePatternSelect} />
```

### Working with RDF/Turtle

```typescript
import { parseTurtle, serializeToTurtle } from '@/lib/rdf/turtle-parser';

// Parse Turtle string
const ontology = await parseTurtle(`
  @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

  :MyWorkflow a yawl:Workflow ;
    yawl:hasTask :Task1 .
`);

// Serialize back to Turtle
const turtleString = await serializeToTurtle(ontology);
```

### Validating Workflows

```typescript
import { validateWorkflow } from '@/lib/validation/pattern-validator';
import { useEditorStore } from '@/store/editor-store';

const workflow = useEditorStore((state) => state.workflow);

if (workflow) {
  const result = validateWorkflow(workflow);

  if (!result.valid) {
    console.error('Validation errors:', result.errors);
  }
}
```

### Using the Editor Store

```typescript
import { useEditorStore } from '@/store/editor-store';

function MyComponent() {
  const { workflow, addNode, selectNodes } = useEditorStore();

  const handleAddNode = () => {
    addNode({
      id: 'node-' + Date.now(),
      type: 'task',
      label: 'New Task',
      position: { x: 100, y: 100 },
    });
  };

  return (
    <button onClick={handleAddNode}>
      Add Node
    </button>
  );
}
```

## Testing

### Unit Tests

```bash
npm run test
```

### Type Checking

```bash
npm run type-check
```

### Linting

```bash
npm run lint
```

### Chicago TDD Tests

```bash
npm run test:chicago
```

## Building for Production

```bash
# Build the application
npm run build

# Start production server
npm run start
```

The production build will be optimized and ready for deployment.

## Common Tasks

### Adding a New shadcn/ui Component

```bash
# Example: Add a Button component
npx shadcn@latest add button
```

This will add the component to `src/components/ui/`.

### Customizing Tailwind Theme

Edit `tailwind.config.ts`:

```typescript
export default {
  theme: {
    extend: {
      colors: {
        'my-color': '#ff0000',
      },
    },
  },
};
```

### Adding Environment Variables

1. Create `.env.local`:

```bash
NEXT_PUBLIC_API_URL=http://localhost:3000/api
```

2. Use in code:

```typescript
const apiUrl = process.env.NEXT_PUBLIC_API_URL;
```

Note: Only variables prefixed with `NEXT_PUBLIC_` are exposed to the browser.

## Troubleshooting

### Port Already in Use

```bash
# Kill process on port 3000
npx kill-port 3000

# Or use a different port
npm run dev -- -p 3001
```

### Module Not Found Errors

```bash
# Clear Next.js cache
rm -rf .next

# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install
```

### TypeScript Errors

```bash
# Restart TypeScript server in your editor
# Or regenerate types
npm run type-check
```

## Next Steps

1. **Read the Architecture Guide**: See `ARCHITECTURE.md` for system design
2. **Review DOCTRINE Covenants**: See `DOCTRINE_COVENANT.md` for development principles
3. **Explore the API**: Check out `src/lib/` for core utilities
4. **Build Your First Workflow**: Open the editor and start creating!

## Resources

- [Next.js Documentation](https://nextjs.org/docs)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [Tailwind CSS](https://tailwindcss.com/docs)
- [shadcn/ui](https://ui.shadcn.com/)
- [Zustand](https://zustand-demo.pmnd.rs/)
- [N3 Library](https://github.com/rdfjs/N3.js)
- [OpenTelemetry JavaScript](https://opentelemetry.io/docs/languages/js/)

## Getting Help

- Check the [KNHK repository](https://github.com/ruvnet/knhk) for issues
- Review the documentation files in this project
- Consult the DOCTRINE covenant documents
