# YAWL UI - Next.js Edition

A modern, production-ready implementation of YAWL (Yet Another Workflow Language) workflow management system using Next.js, shadcn/ui, and RDF/Turtle support.

## Features

✨ **Core Features**
- 🎨 Beautiful, responsive UI with shadcn/ui components
- 📊 Interactive workflow visualization and editing
- 🔀 Support for all YAWL control flow patterns
- 📄 RDF/Turtle import and export
- 🔍 Pattern validation and compliance checking
- 📈 Workflow monitoring and case tracking
- 🎯 Resource management and task allocation
- 🔌 Extensible architecture with plugins

## Tech Stack

- **Framework**: Next.js 16+ (App Router)
- **UI Components**: shadcn/ui + Radix UI
- **Styling**: Tailwind CSS + CSS-in-JS
- **State Management**: Zustand + TanStack Query
- **RDF Processing**: N3.js for Turtle parsing
- **Visualization**: React Flow (prepared for integration)
- **Type Safety**: TypeScript 5+
- **Theming**: next-themes with dark mode support

## Project Structure

```
yawl-ui-nextjs/
├── app/                    # Next.js App Router pages
│   ├── page.tsx           # Home page
│   ├── editor/            # Workflow editor
│   ├── workflows/         # Workflow library
│   ├── monitoring/        # Case monitoring
│   ├── docs/              # Documentation
│   ├── layout.tsx         # Root layout
│   └── globals.css        # Global styles
├── components/            # React components
│   ├── ui/                # shadcn/ui components
│   ├── header.tsx         # Header with navigation
│   ├── sidebar.tsx        # Navigation sidebar
│   └── providers.tsx      # App providers
├── lib/                   # Utilities and services
│   ├── utils.ts          # Helper functions
│   ├── rdf-service.ts    # RDF/Turtle handling
│   └── workflow-service.ts # Workflow logic
├── types/                 # TypeScript type definitions
│   └── yawl.ts           # YAWL types and interfaces
├── public/                # Static assets
└── docs/                  # Project documentation
```

## Getting Started

### Prerequisites

- Node.js 18+
- npm or yarn

### Installation

```bash
# Clone the repository
git clone https://github.com/seanchatmangpt/knhk
cd yawl-ui-nextjs

# Install dependencies
npm install

# Start development server
npm run dev
```

Visit `http://localhost:3000` to see the application.

## Available Scripts

```bash
# Development server with hot reload
npm run dev

# Build for production
npm run build

# Start production server
npm run start

# Type checking
npm run type-check

# Linting (when configured)
npm run lint

# Run tests (coming soon)
npm run test
```

## Key Components

### Workflow Editor
- Create and edit workflow specifications
- Add/remove tasks
- Define control flows
- Validate workflow patterns
- Export to JSON/Turtle

### Workflow Library
- Pre-built workflow templates
- Common patterns and use cases
- Quick-start workflows

### Pattern Validation
- Validates against YAWL control flow patterns
- Checks for orphaned tasks
- Ensures pattern compliance
- Provides validation reports

### RDF/Turtle Support
- Parse Turtle RDF files
- Serialize workflows to RDF
- Semantic web integration
- Linked data compatibility

### Monitoring Dashboard
- View active workflow cases
- Track work item status
- Monitor progress
- View case details

## YAWL Concepts

### Specification
A workflow definition containing tasks, control flows, and data mappings.

### Task
An atomic unit of work that can be assigned to humans or executed automatically.

### Case (Instance)
A running instance of a workflow specification with its own state and data.

### Work Item
An individual task instance requiring action.

### Control Flow Patterns
- **Sequence**: Tasks execute sequentially
- **Parallel**: Multiple tasks execute simultaneously
- **Choice**: One path is selected based on conditions
- **Synchronization**: Wait for multiple tasks to complete
- **And many more...**

## RDF/Turtle Integration

Workflows can be serialized to and parsed from RDF/Turtle format:

```turtle
@prefix yawl: <http://www.yawlfoundation.org/yawl/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

yawl:specification/order-process
  a yawl:Specification ;
  rdfs:label "Order Processing" ;
  yawl:version "1.0" ;
  yawl:hasTask yawl:task/receive-order .
```

## Documentation

- [Architecture Design](../../docs/NEXTJS_YAWL_UI_ARCHITECTURE.md)
- [Quick Start Guide](../../docs/NEXTJS_YAWL_UI_QUICKSTART.md)
- [API Reference](../../docs/NEXTJS_YAWL_UI_SUMMARY.md)

## Contributing

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Commit changes: `git commit -m 'Add feature'`
3. Push branch: `git push origin feature/your-feature`
4. Create Pull Request

## License

MIT License - see LICENSE file for details

## Support

For questions and issues, please refer to the [project documentation](../../docs/) or create an issue in the repository.

## Related Resources

- [YAWL Foundation](http://www.yawlfoundation.org/)
- [YAWL UI Original Repository](https://github.com/yawlfoundation/yawlui)
- [Next.js Documentation](https://nextjs.org/docs)
- [shadcn/ui Components](https://ui.shadcn.com/)
- [RDF/Turtle Specification](https://www.w3.org/TR/turtle/)
