//! HTML Report Generation
//!
//! Generates styled HTML reports for validation results.

use crate::{ValidationReport, ValidationResult, ValidationCategory};
use std::io::Write;

/// Generate HTML report from validation report
pub fn generate_html_report(report: &ValidationReport) -> String {
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n");
    html.push_str("<head>\n");
    html.push_str("  <title>DoD Validation Report</title>\n");
    html.push_str("  <style>\n");
    html.push_str("    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }\n");
    html.push_str("    .summary { background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }\n");
    html.push_str("    .summary h1 { margin: 0 0 20px 0; color: #333; }\n");
    html.push_str("    .stats { display: flex; gap: 20px; flex-wrap: wrap; }\n");
    html.push_str("    .stat { padding: 10px 15px; background: #f0f0f0; border-radius: 4px; }\n");
    html.push_str("    .stat.passed { background: #d4edda; color: #155724; }\n");
    html.push_str("    .stat.failed { background: #f8d7da; color: #721c24; }\n");
    html.push_str("    .results { display: flex; flex-direction: column; gap: 20px; }\n");
    html.push_str("    .category { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }\n");
    html.push_str("    .category h2 { margin: 0 0 15px 0; color: #333; border-bottom: 2px solid #eee; padding-bottom: 10px; }\n");
    html.push_str("    .result { margin-bottom: 20px; padding: 15px; border-left: 4px solid #ccc; background: #fafafa; }\n");
    html.push_str("    .result.passed { border-left-color: #28a745; }\n");
    html.push_str("    .result.failed { border-left-color: #dc3545; }\n");
    html.push_str("    .result .header { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }\n");
    html.push_str("    .result .status { font-size: 1.2em; font-weight: bold; }\n");
    html.push_str("    .result.passed .status { color: #28a745; }\n");
    html.push_str("    .result.failed .status { color: #dc3545; }\n");
    html.push_str("    .result .code-snippet { margin: 10px 0; padding: 10px; background: #282c34; border-radius: 4px; }\n");
    html.push_str("    .result .code-snippet pre { margin: 0; color: #abb2bf; font-family: 'Monaco', 'Menlo', monospace; font-size: 0.9em; }\n");
    html.push_str("    .result .context { margin: 10px 0; padding: 10px; background: #f8f8f8; border-radius: 4px; border: 1px solid #e0e0e0; }\n");
    html.push_str("    .result .context pre { margin: 0; color: #333; font-family: 'Monaco', 'Menlo', monospace; font-size: 0.9em; }\n");
    html.push_str("  </style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    
    // Summary section
    html.push_str("  <div class=\"summary\">\n");
    html.push_str("    <h1>KNHK DoD Validator Report</h1>\n");
    html.push_str(&format!("    <div class=\"stats\">\n"));
    html.push_str(&format!("      <div class=\"stat\"><span class=\"label\">Total:</span> <span class=\"value\">{}</span></div>\n", report.total));
    html.push_str(&format!("      <div class=\"stat passed\"><span class=\"label\">Passed:</span> <span class=\"value\">{}</span></div>\n", report.passed));
    html.push_str(&format!("      <div class=\"stat failed\"><span class=\"label\">Failed:</span> <span class=\"value\">{}</span></div>\n", report.failed));
    html.push_str(&format!("      <div class=\"stat\"><span class=\"label\">Duration:</span> <span class=\"value\">{}ms</span></div>\n", report.duration_ms));
    html.push_str("    </div>\n");
    html.push_str("  </div>\n");
    
    // Results by category
    html.push_str("  <div class=\"results\">\n");
    for (category, results) in &report.category_results {
        if results.is_empty() {
            continue;
        }
        
        html.push_str(&format!("    <div class=\"category\">\n"));
        html.push_str(&format!("      <h2>{:?}</h2>\n", category));
        
        for result in results {
            let status_class = if result.passed { "passed" } else { "failed" };
            let status_symbol = if result.passed { "✓" } else { "✗" };
            
            html.push_str(&format!("      <div class=\"result {}\">\n", status_class));
            html.push_str(&format!("        <div class=\"header\">\n"));
            html.push_str(&format!("          <span class=\"status\">{}</span>\n", status_symbol));
            html.push_str(&format!("          <span class=\"message\">{}</span>\n", escape_html(&result.message)));
            html.push_str("        </div>\n");
            
            if let Some(ref file) = result.file {
                html.push_str(&format!("        <div class=\"file\">File: {}</div>\n", escape_html(&file.display().to_string())));
            }
            
            if let Some(line) = result.line {
                if let Some(col) = result.column {
                    html.push_str(&format!("        <div class=\"location\">Line {}, Column {}</div>\n", line, col));
                } else {
                    html.push_str(&format!("        <div class=\"location\">Line {}</div>\n", line));
                }
            }
            
            if let Some(ref snippet) = result.code_snippet {
                html.push_str("        <div class=\"code-snippet\">\n");
                html.push_str("          <pre><code>");
                html.push_str(&escape_html(snippet));
                html.push_str("</code></pre>\n");
                html.push_str("        </div>\n");
            }
            
            if let Some(ref context) = result.context_lines {
                if !context.is_empty() {
                    html.push_str("        <div class=\"context\">\n");
                    html.push_str("          <pre><code>");
                    for (idx, ctx_line) in context.iter().enumerate() {
                        let line_num = if let Some(line) = result.line {
                            let context_start = line.saturating_sub(3);
                            context_start + idx as u32
                        } else {
                            idx as u32 + 1
                        };
                        
                        html.push_str(&format!("{:4}| {}\n", line_num, escape_html(ctx_line)));
                    }
                    html.push_str("</code></pre>\n");
                    html.push_str("        </div>\n");
                }
            }
            
            if let Some(span_id) = result.span_id {
                html.push_str(&format!("        <div class=\"metadata\">Span ID: 0x{:x}</div>\n", span_id));
            }
            
            if let Some(duration) = result.duration_ns {
                html.push_str(&format!("        <div class=\"metadata\">Duration: {}ns</div>\n", duration));
            }
            
            html.push_str("      </div>\n");
        }
        
        html.push_str("    </div>\n");
    }
    
    html.push_str("  </div>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");
    
    html
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

