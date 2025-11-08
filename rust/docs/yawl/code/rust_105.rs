// ✅ Good: Multiple notifications
let pattern = MultiChoicePattern::new(vec![
    (needs_email, email_notifier),
    (needs_sms, sms_notifier),
    (needs_slack, slack_notifier),
])?;