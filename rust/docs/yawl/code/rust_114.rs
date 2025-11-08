#[cfg(target_os = "linux")]
pub fn measure<F, R>(&self, name: &str, mut f: F) -> BenchmarkResult
where
    F: FnMut() -> R,
{
    // Setup hardware performance counters
    let mut group = Group::new()?;
    let cycles_counter = Builder::new()
        .group(&mut group)
        .kind(Hardware::CPU_CYCLES)
        .build()?;

    let instrs_counter = Builder::new()
        .group(&mut group)
        .kind(Hardware::INSTRUCTIONS)
        .build()?;

    let cache_refs_counter = Builder::new()
        .kind(Hardware::CACHE_REFERENCES)
        .build()?;

    // ... measure with hardware counters

    println!("  {:.2} cycles/op", cycles / iterations);
    println!("  {:.2} instructions/op", instrs / iterations);
    println!("  {:.2} IPC", instrs / cycles);
    println!("  {:.2}% cache miss rate", cache_misses / cache_refs * 100.0);
}