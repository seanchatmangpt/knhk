# KNHK Compilation Performance Benchmark Report

**Generated:** 2025-11-07 18:14:32

**Total Packages:** 13

## Summary

| Package | LOC | Files | Dependencies | Clean Build | Incremental | Test Build |
|---------|----:|------:|--------------|------------:|------------:|-----------:|
| `knhk-unrdf` | 5,310 | 25 | 0/700 | 2m 32.3s | 2m 27.7s | 4.51s |
| `knhk-aot` | 921 | 7 | 0/4 | 1m 33.1s | 2.48s | 1.12s |
| `knhk-lockchain` | 1,097 | 4 | 0/140 | 1m 8.4s | 1.37s | 34.37s |
| `knhk-cli` | 3,496 | 32 | 0/715 | 58.95s | 1m 40.8s | 34.96s |
| `knhk-etl` | 7,877 | 27 | 0/623 | 48.01s | 17.75s | 5.20s |
| `knhk-otel` | 1,577 | 2 | 0/264 | 22.12s | 2.04s | 1m 40.7s |
| `knhk-integration-tests` | 138 | 1 | 0/913 | 21.40s | 27.03s | 12.21s |
| `knhk-warm` | 2,054 | 11 | 0/631 | 21.36s | 28.39s | 1m 9.9s |
| `knhk-patterns` | 2,918 | 9 | 0/643 | 20.63s | 990ms | 6.61s |
| `knhk-validation` | 2,594 | 10 | 0/362 | 15.08s | 336ms | 1.75s |
| `knhk-hot` | 1,867 | 8 | 0/6 | 11.29s | 2.54s | 1.05s |
| `knhk-connectors` | 2,378 | 3 | 0/1 | 897ms | 1.31s | 1m 29.6s |
| `knhk-config` | 544 | 4 | 0/31 | 269ms | 2.18s | 832ms |

## Performance Profiles

### Slowest to Build (Clean)

1. **knhk-unrdf**: 2m 32.3s
2. **knhk-aot**: 1m 33.1s
3. **knhk-lockchain**: 1m 8.4s
4. **knhk-cli**: 58.95s
5. **knhk-etl**: 48.01s

### Largest Codebases

1. **knhk-etl**: 7,877 lines in 27 files
2. **knhk-unrdf**: 5,310 lines in 25 files
3. **knhk-cli**: 3,496 lines in 32 files
4. **knhk-patterns**: 2,918 lines in 9 files
5. **knhk-validation**: 2,594 lines in 10 files

## Optimization Recommendations

1. **Slow builds**: Focus on packages with >20s clean build times
2. **High dependencies**: Review packages with >100 transitive dependencies
3. **Large codebases**: Consider splitting packages >3000 LOC
4. **Incremental builds**: Improve packages where incremental is >30% of clean
