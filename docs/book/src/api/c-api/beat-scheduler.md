# Beat Scheduler C API

Complete C API reference for beat scheduler.

## Functions

### knhk_beat_next

```c
uint64_t knhk_beat_next(void);
```

Advance cycle counter atomically and return new cycle value.

**Returns**: New cycle value

**Thread Safety**: Thread-safe (atomic operation)

### knhk_beat_tick

```c
static inline uint64_t knhk_beat_tick(uint64_t cycle);
```

Extract tick (0-7) from cycle value.

**Parameters**:
- `cycle`: Cycle value

**Returns**: Tick value (0-7)

**Implementation**: `cycle & 0x7` (branchless)

### knhk_beat_pulse

```c
static inline uint64_t knhk_beat_pulse(uint64_t cycle);
```

Compute pulse signal (1 when tick==0, 0 otherwise).

**Parameters**:
- `cycle`: Cycle value

**Returns**: Pulse value (1 or 0)

**Implementation**: `!(cycle & 0x7)` (branchless)

## Header File

`c/include/knhk/beat.h`

## Related Documentation

- [C API](../c-api.md) - Overview
- [Beat Scheduler](../../architecture/8beat-system/beat-scheduler.md) - Architecture
