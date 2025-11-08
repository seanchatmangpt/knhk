# Fiber C API

Complete C API reference for fiber execution.

## Functions

### knhk_fiber_execute

```c
knhk_fiber_result_t knhk_fiber_execute(knhk_fiber_t *fiber, uint64_t tick);
```

Execute μ on ≤8 items at tick slot.

**Parameters**:
- `fiber`: Fiber structure
- `tick`: Tick slot (0-7)

**Returns**: Fiber result structure

### knhk_fiber_park

```c
int knhk_fiber_park(knhk_fiber_t *fiber, knhk_work_t *work);
```

Park over-budget work to W1.

**Parameters**:
- `fiber`: Fiber structure
- `work`: Work to park

**Returns**: 0 on success, -1 on error

### knhk_fiber_process_tick

```c
int knhk_fiber_process_tick(knhk_fiber_t *fiber, uint64_t tick);
```

Process tick (read → execute → write).

**Parameters**:
- `fiber`: Fiber structure
- `tick`: Tick slot (0-7)

**Returns**: 0 on success, -1 on error

## Header File

`c/include/knhk/fiber.h`

## Related Documentation

- [C API](../c-api.md) - Overview
- [Fiber Execution](../../architecture/fiber-execution.md) - Architecture
