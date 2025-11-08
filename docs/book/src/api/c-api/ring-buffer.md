# Ring Buffer C API

Complete C API reference for ring buffers.

## Delta Ring Functions

### knhk_ring_enqueue_delta

```c
int knhk_ring_enqueue_delta(knhk_delta_ring_t *ring, uint64_t tick, knhk_delta_t *delta);
```

Enqueue delta to Δ-ring at tick slot.

**Parameters**:
- `ring`: Ring buffer structure
- `tick`: Tick slot (0-7)
- `delta`: Delta to enqueue

**Returns**: 0 on success, -1 on error

### knhk_ring_dequeue_delta

```c
int knhk_ring_dequeue_delta(knhk_delta_ring_t *ring, uint64_t tick, knhk_delta_t *delta);
```

Dequeue delta from Δ-ring at tick slot.

**Parameters**:
- `ring`: Ring buffer structure
- `tick`: Tick slot (0-7)
- `delta`: Output delta

**Returns**: 0 on success, -1 on error

## Assertion Ring Functions

### knhk_ring_commit_assertion

```c
int knhk_ring_commit_assertion(knhk_assertion_ring_t *ring, uint64_t tick, knhk_assertion_t *assertion);
```

Commit assertion to A-ring at tick slot.

**Parameters**:
- `ring`: Ring buffer structure
- `tick`: Tick slot (0-7)
- `assertion`: Assertion to commit

**Returns**: 0 on success, -1 on error

## Header File

`c/include/knhk/ring.h`

## Related Documentation

- [C API](../c-api.md) - Overview
- [Ring Buffers](../../architecture/ring-buffers.md) - Architecture
