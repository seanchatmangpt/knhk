use knhk_workflow_engine::integration::otel::OtelIntegration;

let otel = OtelIntegration::new()?;
// Spans are automatically created for workflow execution