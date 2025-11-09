#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Workflow visualization module
//!
//! Generates visual diagrams from workflow specifications using GraphViz/DOT format.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use std::collections::HashMap;

/// Workflow visualizer
pub struct WorkflowVisualizer {
    /// Node styling options
    node_styles: HashMap<String, NodeStyle>,
}

/// Node styling configuration
#[derive(Debug, Clone)]
pub struct NodeStyle {
    /// Shape (box, ellipse, diamond, etc.)
    pub shape: String,
    /// Fill color
    pub fillcolor: String,
    /// Border color
    pub color: String,
    /// Label
    pub label: String,
}

impl WorkflowVisualizer {
    /// Create a new workflow visualizer
    pub fn new() -> Self {
        let mut node_styles = HashMap::new();

        // Default styles for different node types
        node_styles.insert(
            "start".to_string(),
            NodeStyle {
                shape: "ellipse".to_string(),
                fillcolor: "#90EE90".to_string(),
                color: "#006400".to_string(),
                label: "Start".to_string(),
            },
        );

        node_styles.insert(
            "end".to_string(),
            NodeStyle {
                shape: "ellipse".to_string(),
                fillcolor: "#FFB6C1".to_string(),
                color: "#8B0000".to_string(),
                label: "End".to_string(),
            },
        );

        node_styles.insert(
            "task".to_string(),
            NodeStyle {
                shape: "box".to_string(),
                fillcolor: "#87CEEB".to_string(),
                color: "#000080".to_string(),
                label: "Task".to_string(),
            },
        );

        node_styles.insert(
            "condition".to_string(),
            NodeStyle {
                shape: "diamond".to_string(),
                fillcolor: "#DDA0DD".to_string(),
                color: "#4B0082".to_string(),
                label: "Condition".to_string(),
            },
        );

        Self { node_styles }
    }

    /// Generate DOT graph from workflow specification
    pub fn generate_dot(&self, spec: &WorkflowSpec) -> WorkflowResult<String> {
        let mut dot = String::from("digraph workflow {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=box, style=rounded];\n\n");

        // Add start node
        if let Some(ref _start_id) = spec.start_condition {
            dot.push_str(&format!(
                "    start [shape=ellipse, fillcolor=\"#90EE90\", label=\"Start\"];\n"
            ));
        }

        // Add tasks
        for (task_id, task) in &spec.tasks {
            let label = format!("{}\\n({:?})", task.name, task.split_type);
            dot.push_str(&format!(
                "    \"{}\" [label=\"{}\", fillcolor=\"#87CEEB\"];\n",
                task_id, label
            ));
        }

        // Add conditions
        for (condition_id, condition) in &spec.conditions {
            dot.push_str(&format!(
                "    \"{}\" [shape=diamond, fillcolor=\"#DDA0DD\", label=\"{}\"];\n",
                condition_id, condition.name
            ));
        }

        // Add end node
        if let Some(ref _end_id) = spec.end_condition {
            dot.push_str(&format!(
                "    end [shape=ellipse, fillcolor=\"#FFB6C1\", label=\"End\"];\n"
            ));
        }

        dot.push_str("\n");

        // Add edges from start
        if let Some(ref start_id) = spec.start_condition {
            if let Some(start_condition) = spec.conditions.get(start_id) {
                for task_id in &start_condition.outgoing_flows {
                    dot.push_str(&format!("    start -> \"{}\";\n", task_id));
                }
            }
        }

        // Add task edges
        for (task_id, task) in &spec.tasks {
            for output_condition_id in &task.output_conditions {
                if let Some(output_condition) = spec.conditions.get(output_condition_id) {
                    for next_task_id in &output_condition.outgoing_flows {
                        dot.push_str(&format!("    \"{}\" -> \"{}\";\n", task_id, next_task_id));
                    }
                }
            }

            // Direct task-to-task flows
            for next_task_id in &task.outgoing_flows {
                dot.push_str(&format!("    \"{}\" -> \"{}\";\n", task_id, next_task_id));
            }
        }

        // Add edges to end
        if let Some(ref end_id) = spec.end_condition {
            if let Some(end_condition) = spec.conditions.get(end_id) {
                for task_id in &end_condition.incoming_flows {
                    dot.push_str(&format!("    \"{}\" -> end;\n", task_id));
                }
            }
        }

        dot.push_str("}\n");

        Ok(dot)
    }

    /// Generate SVG from DOT (requires external GraphViz)
    pub fn render_svg(&self, _dot_content: &str) -> WorkflowResult<String> {
        // In production, would call GraphViz `dot` command
        // For now, return error indicating GraphViz is required
        Err(WorkflowError::Internal(
            "SVG rendering requires GraphViz. Install with: brew install graphviz".to_string(),
        ))
    }

    /// Generate interactive HTML diagram
    pub fn generate_html(&self, spec: &WorkflowSpec) -> WorkflowResult<String> {
        let dot = self.generate_dot(spec)?;

        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Workflow: {}</title>
    <script src="https://cdn.jsdelivr.net/npm/viz.js@2.1.2/viz.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/viz.js@2.1.2/full.render.js"></script>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 20px;
        }}
        #diagram {{
            border: 1px solid #ccc;
            padding: 20px;
            background: white;
        }}
        .info {{
            margin-top: 20px;
            padding: 10px;
            background: #f0f0f0;
            border-radius: 5px;
        }}
    </style>
</head>
<body>
    <h1>Workflow: {}</h1>
    <div id="diagram"></div>
    <div class="info">
        <h3>Workflow Information</h3>
        <p><strong>Tasks:</strong> {}</p>
        <p><strong>Conditions:</strong> {}</p>
    </div>
    <script>
        var viz = new Viz();
        var dot = `{}`;
        viz.renderSVGElement(dot)
            .then(function(element) {{
                document.getElementById('diagram').appendChild(element);
            }})
            .catch(error => {{
                console.error(error);
                document.getElementById('diagram').innerHTML = '<p>Error rendering diagram. Please install GraphViz.</p>';
            }});
    </script>
</body>
</html>
"#,
            spec.name,
            spec.name,
            spec.tasks.len(),
            spec.conditions.len(),
            dot
        );

        Ok(html)
    }
}

impl Default for WorkflowVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{JoinType, SplitType, Task, TaskType, WorkflowSpec, WorkflowSpecId};
    use std::collections::HashMap;

    #[test]
    fn test_generate_dot() {
        let visualizer = WorkflowVisualizer::new();

        let mut spec = WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: "Test Workflow".to_string(),
            tasks: HashMap::new(),
            conditions: HashMap::new(),
            start_condition: None,
            end_condition: None,
            source_turtle: None,
        };

        let dot = visualizer.generate_dot(&spec).unwrap();
        assert!(dot.contains("digraph workflow"));
    }
}
