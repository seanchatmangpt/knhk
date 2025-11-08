// Check resource availability
let available = engine.resource_allocator()
    .is_available(resource_id)
    .await?;