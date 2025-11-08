# List available templates
knhk-workflow templates list

# Show template details
knhk-workflow templates show two-stage-approval

# Instantiate template
knhk-workflow templates instantiate two-stage-approval \
    --params '{"approver1_role": "manager"}'