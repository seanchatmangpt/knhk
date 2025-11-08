// Update resource workload
engine.resource_allocator()
    .update_workload(resource_id, 1)
    .await?;

// Get resource availability
let available = engine.resource_allocator()
    .is_available(resource_id)
    .await?;