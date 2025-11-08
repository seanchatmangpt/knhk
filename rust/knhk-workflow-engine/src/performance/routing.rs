#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Hot/warm/cold path routing for performance optimization

// Unused import removed - will be used when implementing routing
use crate::patterns::PatternId;
use crate::resilience::PathType;

/// Routing decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingDecision {
    /// Route to hot path (≤8 ticks)
    Hot,
    /// Route to warm path (≤500 µs)
    Warm,
    /// Route to cold path (≤200 ms)
    Cold,
}

/// Path router for performance optimization
pub struct PathRouter {
    /// Pattern to path type mapping
    pattern_paths: std::collections::HashMap<PatternId, PathType>,
}

impl PathRouter {
    /// Create a new path router
    pub fn new() -> Self {
        let mut router = Self {
            pattern_paths: std::collections::HashMap::new(),
        };

        // Initialize pattern routing (basic patterns go to hot path)
        for id in 1..=5 {
            if let Ok(pattern_id) = PatternId::new(id) {
                router.pattern_paths.insert(pattern_id, PathType::Hot);
            }
        }

        // Advanced patterns go to warm path
        for id in 6..=25 {
            if let Ok(pattern_id) = PatternId::new(id) {
                router.pattern_paths.insert(pattern_id, PathType::Warm);
            }
        }

        // Complex patterns go to cold path
        for id in 26..=43 {
            if let Ok(pattern_id) = PatternId::new(id) {
                router.pattern_paths.insert(pattern_id, PathType::Cold);
            }
        }

        router
    }

    /// Route a pattern to appropriate path
    pub fn route_pattern(&self, pattern_id: &PatternId) -> RoutingDecision {
        self.pattern_paths
            .get(pattern_id)
            .map(|path_type| match path_type {
                PathType::Hot => RoutingDecision::Hot,
                PathType::Warm => RoutingDecision::Warm,
                PathType::Cold => RoutingDecision::Cold,
            })
            .unwrap_or(RoutingDecision::Warm) // Default to warm
    }

    /// Check if pattern can be routed to hot path
    pub fn can_route_hot(&self, pattern_id: &PatternId) -> bool {
        matches!(self.route_pattern(pattern_id), RoutingDecision::Hot)
    }

    /// Update pattern routing
    pub fn update_routing(&mut self, pattern_id: PatternId, path_type: PathType) {
        self.pattern_paths.insert(pattern_id, path_type);
    }
}

impl Default for PathRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_router() {
        let router = PathRouter::new();
        let pattern_id = PatternId::new(1).unwrap();
        assert_eq!(router.route_pattern(&pattern_id), RoutingDecision::Hot);
    }
}
