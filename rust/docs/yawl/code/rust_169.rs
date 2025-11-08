let detector = DeadlockDetector;
let result = detector.validate(&spec)?;
println!("Cycles: {:?}", result.cycles);