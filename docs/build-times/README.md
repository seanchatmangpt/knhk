# Build Time Tracking

This directory contains build time measurements and documentation for the KNHK workspace.

## Files

- `BUILD_TIMES.md` - Human-readable build time documentation
- `build-times.json` - Machine-readable JSON data (updated by measurement script)
- `build-times.csv` - CSV data for analysis (updated by measurement script)

## Usage

### View Current Build Times

Read `BUILD_TIMES.md` for a quick reference of build times.

### Measure Build Times

Run the measurement script to get current build times on your system:

```bash
make measure-build-times
# or
./scripts/measure-build-times.sh
```

This will:
1. Clean build each crate
2. Measure debug, release, test, and check times
3. Update `BUILD_TIMES.md`, `build-times.json`, and `build-times.csv`

**Note**: Full measurement takes ~10-15 minutes.

### Analyze Historical Trends

Use the CSV file to analyze build time trends over time:

```bash
# View CSV data
cat build-times.csv

# Analyze with your favorite tool (Python, R, etc.)
```

## Understanding Build Times

### Clean vs Incremental Builds

- **Clean builds**: Full rebuild from scratch (what the script measures)
- **Incremental builds**: Only rebuild changed code (typical development)

Incremental builds are typically **5-10x faster** than clean builds.

### Build Time Factors

Build times depend on:
- **Code size**: More LOC = longer build
- **Dependencies**: Heavy deps (tokio, clap) = longer build
- **System**: CPU cores, RAM, disk speed
- **Cargo version**: Newer versions often faster

### Typical Build Times

| Crate Size | Debug Build | Release Build |
|------------|-------------|---------------|
| Small (< 1k LOC) | 1-3s | 3-10s |
| Medium (1-5k LOC) | 3-10s | 10-60s |
| Large (5-15k LOC) | 10-30s | 60-180s |
| Very Large (> 15k LOC) | 30-90s | 180-300s |

## Optimization Tips

See `BUILD_TIMES.md` for optimization tips and development workflow recommendations.

