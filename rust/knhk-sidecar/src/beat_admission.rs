// rust/knhk-sidecar/src/beat_admission.rs
// Beat-driven admission for 8-beat epoch system
// Replaces direct ETL pipeline calls with beat-aligned admission

use std::sync::{Arc, Mutex};
use knhk_etl::beat_scheduler::{BeatScheduler, BeatSchedulerError};
use knhk_etl::ingest::RawTriple;
use crate::error::{SidecarError, SidecarResult};

/// Beat admission manager
/// Handles delta admission with cycle_id stamping and beat alignment
pub struct BeatAdmission {
    /// Beat scheduler (shared across all admission requests)
    beat_scheduler: Arc<Mutex<BeatScheduler>>,
    /// Default domain ID for admission (0 = default domain)
    default_domain_id: usize,
}

impl BeatAdmission {
    /// Create new beat admission manager
    pub fn new(
        beat_scheduler: Arc<Mutex<BeatScheduler>>,
        default_domain_id: usize,
    ) -> Self {
        Self {
            beat_scheduler,
            default_domain_id,
        }
    }

    /// Admit delta to beat scheduler
    /// Returns cycle_id for response correlation
    pub fn admit_delta(
        &self,
        delta: Vec<RawTriple>,
        domain_id: Option<usize>,
    ) -> SidecarResult<u64> {
        let domain = domain_id.unwrap_or(self.default_domain_id);
        
        // Get current cycle from beat scheduler
        let current_cycle = self.beat_scheduler
            .lock()
            .map_err(|e| SidecarError::internal_error(
                format!("Failed to acquire beat scheduler lock: {}", e)
            ))?
            .current_cycle();

        // Enqueue delta to delta ring with cycle_id stamping
        self.beat_scheduler
            .lock()
            .map_err(|e| SidecarError::internal_error(
                format!("Failed to acquire beat scheduler lock: {}", e)
            ))?
            .enqueue_delta(domain, delta, current_cycle)
            .map_err(|e| match e {
                BeatSchedulerError::RingBufferFull => {
                    SidecarError::BatchError {
                        context: crate::error::ErrorContext::new(
                            "SIDECAR_BEAT_RING_FULL",
                            "Delta ring buffer is full - backpressure applied"
                        )
                        .with_attribute("domain_id", domain.to_string())
                        .with_attribute("cycle_id", current_cycle.to_string()),
                    }
                }
                BeatSchedulerError::InvalidDomainCount => {
                    SidecarError::config_error(format!("Invalid domain ID: {}", domain))
                }
                _ => SidecarError::internal_error(format!("Beat scheduler error: {:?}", e)),
            })?;

        Ok(current_cycle)
    }

    /// Get current cycle
    pub fn get_current_cycle(&self) -> SidecarResult<u64> {
        Ok(self.beat_scheduler
            .lock()
            .map_err(|e| SidecarError::internal_error(
                format!("Failed to acquire beat scheduler lock: {}", e)
            ))?
            .current_cycle())
    }

    /// Get current tick (0-7)
    pub fn get_current_tick(&self) -> SidecarResult<u64> {
        Ok(self.beat_scheduler
            .lock()
            .map_err(|e| SidecarError::internal_error(
                format!("Failed to acquire beat scheduler lock: {}", e)
            ))?
            .current_tick())
    }

    /// Check if admission should throttle (backpressure)
    pub fn should_throttle(&self, _domain_id: Option<usize>) -> SidecarResult<bool> {
        // Check if delta ring is full
        // Note: RingBuffer doesn't expose is_full() directly, so we check by attempting enqueue
        // In production, would add is_full() method to RingBuffer
        Ok(false) // Placeholder - would check ring buffer capacity
    }

    /// Get park count (number of parked deltas)
    pub fn get_park_count(&self) -> SidecarResult<usize> {
        Ok(self.beat_scheduler
            .lock()
            .map_err(|e| SidecarError::internal_error(
                format!("Failed to acquire beat scheduler lock: {}", e)
            ))?
            .park_count())
    }
}


