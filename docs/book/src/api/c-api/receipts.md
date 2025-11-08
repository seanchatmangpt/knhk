# Receipt C API

Complete C API reference for receipt generation and validation.

## Functions

### knhk_generate_receipt

```c
int knhk_generate_receipt(knhk_operation_t *op, knhk_receipt_t *receipt);
```

Generate receipt for operation.

**Parameters**:
- `op`: Operation structure
- `receipt`: Output receipt structure

**Returns**: 0 on success, -1 on error

### knhk_validate_receipt

```c
int knhk_validate_receipt(knhk_receipt_t *receipt);
```

Validate receipt.

**Parameters**:
- `receipt`: Receipt structure

**Returns**: 0 if valid, -1 if invalid

## Receipt Structure

```c
typedef struct {
    uint64_t span_id;
    uint8_t receipt_hash[32];
    uint8_t operation_hash[32];
} knhk_receipt_t;
```

## Header File

`c/include/knhk/receipts.h`

## Related Documentation

- [C API](../c-api.md) - Overview
- [Lockchain](../../integration/lockchain.md) - Receipt storage
