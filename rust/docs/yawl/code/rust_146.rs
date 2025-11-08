use knhk_workflow_engine::performance::WorkflowProfiler;

let mut profiler = WorkflowProfiler::new();
let metrics = profiler.profile_case(&engine, case_id).await?;
let report = profiler.generate_report(&metrics)?;
let analysis = profiler.analyze_hot_path(&metrics)?;