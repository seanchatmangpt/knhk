//! OTEL Macros for Workflow Engine
//!
//! Provides convenient macros for OpenTelemetry instrumentation following
//! Van der Aalst production OTEL requirements.

/// Macro to start an OTEL span with XES attributes
///
/// Usage:
/// ```rust
/// let span_ctx = otel_span!(
///     otel,
///     "knhk.workflow_engine.execute_task",
///     case_id: Some(&case_id),
///     spec_id: Some(&spec_id),
///     task_id: Some(&task.id),
///     pattern_id: Some(&pattern_id)
/// ).await?;
/// ```
#[macro_export]
macro_rules! otel_span {
    (
        $otel:expr,
        $span_name:expr,
        $(case_id: $case_id:expr,)?
        $(spec_id: $spec_id:expr,)?
        $(task_id: $task_id:expr,)?
        $(pattern_id: $pattern_id:expr,)?
        $(parent: $parent:expr,)?
    ) => {{
        use $crate::integration::otel_helpers::create_trace_context;
        use knhk_otel::SpanContext;

        let mut guard = $otel.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            // Create parent context for trace correlation
            let parent_ctx = {
                let mut ctx = None;
                $(
                    if let Some(parent) = $parent {
                        ctx = Some(parent.clone());
                    }
                )?
                if ctx.is_none() {
                    $(
                        if let Some(cid) = $case_id {
                            ctx = create_trace_context(cid);
                        }
                    )?
                }
                ctx
            };

            let span_ctx = tracer.start_span($span_name.to_string(), parent_ctx);

            // Add XES-compatible attributes
            let timestamp = chrono::Utc::now().to_rfc3339();
            tracer.add_attribute(
                span_ctx.clone(),
                "time:timestamp".to_string(),
                timestamp,
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "lifecycle:transition".to_string(),
                "start".to_string(),
            );

            // Add workflow attributes
            $(
                if let Some(cid) = $case_id {
                    tracer.add_attribute(
                        span_ctx.clone(),
                        "knhk.workflow_engine.case_id".to_string(),
                        cid.to_string(),
                    );
                }
            )?
            $(
                if let Some(sid) = $spec_id {
                    tracer.add_attribute(
                        span_ctx.clone(),
                        "knhk.workflow_engine.spec_id".to_string(),
                        sid.to_string(),
                    );
                }
            )?
            $(
                if let Some(tid) = $task_id {
                    tracer.add_attribute(
                        span_ctx.clone(),
                        "knhk.workflow_engine.task_id".to_string(),
                        tid.to_string(),
                    );
                }
            )?
            $(
                if let Some(pid) = $pattern_id {
                    tracer.add_attribute(
                        span_ctx.clone(),
                        "knhk.workflow_engine.pattern_id".to_string(),
                        pid.0.to_string(),
                    );
                }
            )?

            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }};
}

/// Macro to end an OTEL span with lifecycle transition
///
/// Usage:
/// ```rust
/// otel_span_end!(
///     otel,
///     span_ctx,
///     success: true,
///     start_time: task_start_time
/// ).await?;
/// ```
#[macro_export]
macro_rules! otel_span_end {
    (
        $otel:expr,
        $span_ctx:expr,
        success: $success:expr,
        start_time: $start_time:expr
    ) => {{
        use std::time::Instant;
        use $crate::integration::otel_helpers::end_span_with_result;

        if let Some(ref span) = $span_ctx.as_ref() {
            end_span_with_result($otel, (*span).clone(), $success, $start_time).await?;
        }
        Ok::<(), $crate::error::WorkflowError>(())
    }};

    (
        $otel:expr,
        $span_ctx:expr,
        success: $success:expr,
        latency_ms: $latency_ms:expr
    ) => {{
        use knhk_otel::SpanStatus;

        if let Some(ref span) = $span_ctx.as_ref() {
            $otel
                .add_attribute(
                    (*span).clone(),
                    "knhk.workflow_engine.success".to_string(),
                    $success.to_string(),
                )
                .await?;

            $otel
                .add_attribute(
                    (*span).clone(),
                    "knhk.workflow_engine.latency_ms".to_string(),
                    $latency_ms.to_string(),
                )
                .await?;

            let transition = if $success { "complete" } else { "cancel" };
            $otel
                .add_lifecycle_transition((*span).clone(), transition)
                .await?;

            $otel
                .end_span(
                    (*span).clone(),
                    if $success {
                        SpanStatus::Ok
                    } else {
                        SpanStatus::Error
                    },
                )
                .await?;
        }
        Ok::<(), $crate::error::WorkflowError>(())
    }};
}

/// Macro to add attributes to a span conditionally
///
/// Usage:
/// ```rust
/// otel_attr!(
///     otel,
///     span_ctx,
///     "knhk.workflow_engine.resource_utilization" => utilization.to_string(),
///     "knhk.workflow_engine.bottleneck_detected" => "true"
/// ).await?;
/// ```
#[macro_export]
macro_rules! otel_attr {
    (
        $otel:expr,
        $span_ctx:expr,
        $($key:expr => $value:expr),+ $(,)?
    ) => {{
          if let Some(span) = $span_ctx {
            $(
                $otel.add_attribute(
                    (*span).clone(),
                    $key.to_string(),
                    $value.to_string(),
                )
                .await?;
            )+
        }
        Ok::<(), $crate::error::WorkflowError>(())
    }};
}

/// Macro to execute code within an OTEL span
///
/// Usage:
/// ```rust
/// let result = otel_with_span!(
///     otel,
///     "knhk.workflow_engine.execute_task",
///     case_id: Some(&case_id),
///     task_id: Some(&task.id),
///     {
///         // Your code here
///         execute_task().await
///     }
/// ).await?;
/// ```
#[macro_export]
macro_rules! otel_with_span {
    (
        $otel:expr,
        $span_name:expr,
        $(case_id: $case_id:expr,)?
        $(spec_id: $spec_id:expr,)?
        $(task_id: $task_id:expr,)?
        $(pattern_id: $pattern_id:expr,)?
        $(parent: $parent:expr,)?
        $code:block
    ) => {{
        use std::time::Instant;

        let start_time = Instant::now();
        let span_ctx = $crate::otel_span!(
            $otel,
            $span_name,
            $(case_id: $case_id,)?
            $(spec_id: $spec_id,)?
            $(task_id: $task_id,)?
            $(pattern_id: $pattern_id,)?
            $(parent: $parent,)?
        ).await?;

        let result = $code;

        let success = result.is_ok();
        $crate::otel_span_end!(
            $otel,
            span_ctx,
            success: success,
            start_time: start_time
        ).await?;

        result
    }};
}

/// Macro to add resource attributes to a span
///
/// Usage:
/// ```rust
/// otel_resource!(
///     otel,
///     span_ctx,
///     resource: Some("resource_123"),
///     role: Some("executor")
/// ).await?;
/// ```
#[macro_export]
macro_rules! otel_resource {
    (
        $otel:expr,
        $span_ctx:expr,
        resource: $resource:expr,
        role: $role:expr
    ) => {{
        if let Some(span) = $span_ctx.as_ref() {
            $otel
                .add_resource((*span).clone(), $resource, $role)
                .await?;
        }
        Ok::<(), $crate::error::WorkflowError>(())
    }};
}

/// Macro to add bottleneck detection
///
/// Usage:
/// ```rust
/// otel_bottleneck!(
///     otel,
///     span_ctx,
///     latency_ms: duration_ms,
///     threshold_ms: 1000
/// ).await?;
/// ```
#[macro_export]
macro_rules! otel_bottleneck {
    (
        $otel:expr,
        $span_ctx:expr,
        latency_ms: $latency_ms:expr,
        threshold_ms: $threshold_ms:expr
    ) => {{
        use $crate::integration::otel_helpers::add_bottleneck_if_detected;

        if let Some(ref span) = $span_ctx.as_ref() {
            add_bottleneck_if_detected(
                $otel,
                (*span).clone(),
                $latency_ms as u128,
                $threshold_ms as u128,
            )
            .await?;
        }
        Ok::<(), $crate::error::WorkflowError>(())
    }};
}

/// Macro to add conformance checking attributes
///
/// Usage:
/// ```rust
/// otel_conformance!(
///     otel,
///     span_ctx,
///     expected_pattern: Some(2),
///     actual_pattern: pattern_id.0
/// ).await?;
/// ```
#[macro_export]
macro_rules! otel_conformance {
    (
        $otel:expr,
        $span_ctx:expr,
        expected_pattern: $expected:expr,
        actual_pattern: $actual:expr
    ) => {{
        use $crate::integration::otel_helpers::add_conformance_attributes;

        if let Some(ref span) = $span_ctx.as_ref() {
            add_conformance_attributes($otel, (*span).clone(), $expected, $actual).await?;
        }
        Ok::<(), $crate::error::WorkflowError>(())
    }};
}
