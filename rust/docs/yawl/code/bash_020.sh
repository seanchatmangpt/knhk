# Profile workflow execution
knhk-workflow profile workflow.ttl --case-id <case-id>

# Generate performance report
knhk-workflow profile workflow.ttl --report > performance-report.html

# Export metrics
knhk-workflow profile workflow.ttl --export prometheus > metrics.prom