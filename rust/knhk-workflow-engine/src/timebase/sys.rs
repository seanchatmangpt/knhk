//! Real clock implementation for production

use crate::timebase::trait_impl::Timebase;
use async_trait::async_trait;
use std::time::{Duration, Instant, SystemTime};

/// Real clock implementation for production
#[derive(Clone, Default)]
pub struct SysClock;

#[async_trait]
impl Timebase for SysClock {
    fn now_wall(&self) -> SystemTime {
        SystemTime::now()
    }

    fn now_mono(&self) -> Instant {
        Instant::now()
    }

    fn scale(&self) -> f64 {
        1.0
    }

    async fn sleep(&self, d: Duration) {
        tokio::time::sleep(d).await;
    }

    async fn sleep_until_wall(&self, t: SystemTime) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let tgt = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
        let duration = tgt.saturating_sub(now);
        tokio::time::sleep(duration).await;
    }

    async fn sleep_until_mono(&self, t: Instant) {
        tokio::time::sleep_until(t.into()).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sys_clock() {
        let clock = SysClock;
        let now_wall = clock.now_wall();
        let now_mono = clock.now_mono();

        assert!(now_wall > SystemTime::UNIX_EPOCH);
        assert!(now_mono.elapsed() < Duration::from_secs(1));

        let start = Instant::now();
        clock.sleep(Duration::from_millis(10)).await;
        assert!(start.elapsed() >= Duration::from_millis(10));
    }
}
