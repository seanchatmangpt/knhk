use knhk_workflow_engine::TemplateLibrary;

let library = TemplateLibrary::new();
let template = library.get_template("two-stage-approval")?;
let spec = library.instantiate("two-stage-approval", serde_json::json!({
    "approver1_role": "manager",
    "approver2_role": "director"
}))?;