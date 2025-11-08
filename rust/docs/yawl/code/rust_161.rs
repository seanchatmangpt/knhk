use knhk_workflow_engine::validation::DeadlockDetector;

let detector = DeadlockDetector;
let result = detector.validate(&spec)?;

if !result.cycles.is_empty() {
    println!("Deadlock detected: {:?}", result.cycles);
}