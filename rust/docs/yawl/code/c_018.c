// Real zero-copy ring buffer operations
typedef struct __attribute__((packed, aligned(64))) {
    uint32_t pattern_id;         
    uint32_t algorithm;          
    uint32_t operation;          
    uint64_t key_resource_id;    
    uint64_t timestamp;          
} pqc_crystal_envelope_t;