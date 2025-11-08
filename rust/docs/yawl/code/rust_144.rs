use knhk_workflow_engine::WorkflowVisualizer;

let visualizer = WorkflowVisualizer::new();
let dot_graph = visualizer.generate_dot(&spec)?;
let html = visualizer.generate_html(&spec)?;