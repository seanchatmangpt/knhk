//! RDTSC measurement tests
//!
//! Verifies RDTSC cycle counter accuracy and tick conversion.

use knhk_hot::cycle_counter::{read_cycles, read_cycles_precise, cycles_to_ticks, TickMeasurement};

#[test]
fn test_rdtsc_read_cycles() {
    let cycles = read_cycles();
    assert!(cycles > 0, "RDTSC should return positive cycle count");
}

#[test]
fn test_rdtsc_read_cycles_precise() {
    let cycles = read_cycles_precise();
    assert!(cycles > 0, "RDTSC precise should return positive cycle count");
}

#[test]
fn test_cycles_to_ticks() {
    // At 4GHz, 1 cycle = 0.25ns, so 4 cycles = 1ns = 4 ticks
    let cycles = 4;
    let ticks = cycles_to_ticks(cycles);
    assert_eq!(ticks, 4, "4 cycles should equal 4 ticks at 4GHz");
}

#[test]
fn test_tick_measurement() {
    let measurement = TickMeasurement::new();
    assert!(measurement.ticks() >= 0, "Tick measurement should be non-negative");
}

