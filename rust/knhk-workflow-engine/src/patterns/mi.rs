//! Multiple instance tracker for MI patterns
//!
//! Provides `MITracker` for tracking multiple instance execution state
//! in patterns P12-P15, P22-P25.

use serde_json::json;

/// Multiple instance tracker
#[derive(Debug, Clone)]
pub struct MITracker {
    /// Target instance count (design-time or runtime)
    pub target_n: Option<u32>,
    /// Number of instances launched
    pub launched: u32,
    /// Number of instances completed
    pub completed: u32,
    /// Threshold for completion (for "complete MI" patterns)
    pub threshold_complete: Option<u32>,
}

impl MITracker {
    /// Create a new MI tracker
    pub fn new(target: Option<u32>, threshold: Option<u32>) -> Self {
        Self {
            target_n: target,
            launched: 0,
            completed: 0,
            threshold_complete: threshold,
        }
    }

    /// Launch instances and return their IDs
    pub fn launch(&mut self, how_many: u32) -> Vec<u32> {
        let mut ids = Vec::with_capacity(how_many as usize);
        let start = self.launched;
        for i in 0..how_many {
            ids.push(start + i);
        }
        self.launched += how_many;
        ids
    }

    /// Mark one instance as completed
    pub fn complete_one(&mut self) {
        self.completed += 1;
    }

    /// Check if all instances are done
    pub fn all_done(&self) -> bool {
        match self.target_n {
            Some(n) => self.completed >= n,
            None => false,
        }
    }

    /// Check if threshold is reached
    pub fn threshold_reached(&self) -> bool {
        match self.threshold_complete {
            Some(k) => self.completed >= k,
            None => false,
        }
    }

    /// Convert to JSON update for pattern execution result
    pub fn to_update(&self) -> serde_json::Value {
        json!({
            "target_n": self.target_n,
            "launched": self.launched,
            "completed": self.completed,
            "threshold_complete": self.threshold_complete,
        })
    }
}

impl Default for MITracker {
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mi_launch() {
        let mut mi = MITracker::new(Some(5), None);
        let ids = mi.launch(3);
        assert_eq!(ids, vec![0, 1, 2]);
        assert_eq!(mi.launched, 3);
    }

    #[test]
    fn test_mi_complete() {
        let mut mi = MITracker::new(Some(3), None);
        mi.launch(3);
        assert!(!mi.all_done());

        mi.complete_one();
        assert!(!mi.all_done());

        mi.complete_one();
        mi.complete_one();
        assert!(mi.all_done());
    }

    #[test]
    fn test_mi_threshold() {
        let mut mi = MITracker::new(Some(10), Some(5));
        mi.launch(10);
        assert!(!mi.threshold_reached());

        for _ in 0..5 {
            mi.complete_one();
        }
        assert!(mi.threshold_reached());
    }
}
