# KNHK DFLSS

Design for Lean Six Sigma metrics collection and SPC chart management.

## Overview

`knhk-dflss` provides comprehensive DFLSS (Design For Lean Six Sigma) functionality including:

- **Metrics Collection**: Code quality, performance, Weaver validation
- **SPC Charts**: X-bar/R, p-chart, c-chart with Western Electric rules
- **Process Capability**: Cp, Cpk, Sigma level, DPMO calculations
- **Validation**: CTQ validation and compliance checking
- **Reporting**: Comprehensive DFLSS reports
- **Archiving**: Evidence archiving for certification
- **Innovation Tracking**: TRIZ ideality scores, contradictions, MGPP
- **Fortune 5 Features**: SLO monitoring, promotion gates, multi-region
- **Autonomics**: Reflex map analysis, invariant tracking, self-healing
- **Predictive Analytics**: ML-powered forecasting and anomaly detection
- **Advanced DFLSS Tools**: DOE, Monte Carlo, Taguchi, FMEA, QFD
- **Process Mining**: XES event log analysis, bottleneck detection

## Installation

```bash
cd rust
cargo build --release -p knhk-dflss
```

## Usage

### Basic Commands

```bash
# Collect code quality metrics
knhk-dflss metrics collect-quality --rust-dir rust

# Update X-bar and R charts
knhk-dflss charts update-xbar-r --results perf_results.txt

# Calculate process capability
knhk-dflss capability calculate --data data.csv --usl 8.0

# Validate performance compliance
knhk-dflss validation check-performance --data perf.csv --threshold 8.0
```

### Advanced Commands

```bash
# Innovation tracking
knhk-dflss innovation ideality --version v1.0

# Fortune 5 SLO monitoring
knhk-dflss fortune5 slo monitor --class R1 --window 60

# Autonomics monitoring
knhk-dflss autonomics monitor --duration 3600

# Predictive analytics
knhk-dflss predictive quality --horizon 30 --model lstm

# Process mining
knhk-dflss mining import-xes --file events.xes
```

## Architecture

The CLI is organized into 12 command categories:

1. **metrics** - Metrics collection
2. **charts** - SPC control charts
3. **capability** - Process capability analysis
4. **validation** - CTQ validation
5. **report** - Report generation
6. **archive** - Evidence archiving
7. **innovation** - Innovation tracking
8. **fortune5** - Fortune 5 enterprise features
9. **autonomics** - Autonomics monitoring
10. **predictive** - Predictive analytics
11. **dflss** - Advanced DFLSS tools
12. **mining** - Process mining

## Status

**Current Status**: Basic SPC functionality implemented (metrics collection, X-bar/R charts, process capability). Advanced features (Fortune 5, autonomics, predictive analytics) are planned for future releases.

## License

MIT OR Apache-2.0

