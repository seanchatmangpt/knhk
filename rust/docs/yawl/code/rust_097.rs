let choices = vec![
    (
        Arc::new(|data: &EventData| data.event == Event::A) as ConditionFn<EventData>,
        Arc::new(|mut data: EventData| { data.processed = true; Ok(data) }) as BranchFn<EventData>,
    ),
    (
        Arc::new(|data: &EventData| data.event == Event::B),
        Arc::new(|mut data: EventData| { data.processed = true; Ok(data) }),
    ),
];

let pattern = DeferredChoicePattern::new(choices, 1000)?; // 1000ms timeout
let results = pattern.execute(event_data)?;