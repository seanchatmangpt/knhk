# DoD Validator Documentation

Definition of Done validator using KNHK's 2ns capabilities.

## Overview

The DoD validator provides:
- Code quality validation
- Error handling checks
- Performance validation
- Hot path pattern detection
- Autonomous self-healing

## Architecture

- **Hot Path**: C pattern matching (≤2ns)
- **Warm Path**: Rust orchestration
- **Cold Path**: Complex analysis (unrdf)
- **Autonomous**: Self-healing capabilities

## Key Features

- **Pattern Detection**: Finds unwrap(), TODO, placeholders
- **Performance**: Uses KNHK hot path for ≤2ns checks
- **Autonomous**: Automatic fix generation and application
- **Chicago TDD**: Comprehensive test coverage

## Related Documentation

- [Architecture](../ARCHITECTURE.md) - Validator architecture
- [Autonomics](../AUTONOMICS.md) - Autonomous system design
- [Vision 2027](../VISION_2027.md) - Strategic vision

