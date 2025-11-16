//! Chicago TDD Tests for Work-Stealing Executor
//!
//! Tests work-stealing executor following Chicago-style TDD.

#[cfg(feature = "async-v2")]
mod work_stealing_executor_tests {
    use knhk_workflow_engine::concurrency::{WorkStealingExecutor, WorkerConfig};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;

    mod executor_basic {
        use super::*;

        #[tokio::test]
        async fn test_executor_creation() {
            // Arrange & Act
            let executor = WorkStealingExecutor::new(4);

            // Assert
            assert_eq!(executor.worker_count(), 4);
        }

        #[tokio::test]
        async fn test_executor_with_custom_config() {
            // Arrange
            let config = WorkerConfig {
                worker_count: 8,
                max_steal_attempts: 32,
                park_timeout_ms: 50,
                enable_telemetry: true,
            };

            // Act
            let executor = WorkStealingExecutor::with_config(config);

            // Assert
            assert_eq!(executor.worker_count(), 8);
        }

        #[tokio::test]
        async fn test_spawn_single_task() {
            // Arrange
            let executor = WorkStealingExecutor::new(2);
            let executed = Arc::new(AtomicUsize::new(0));
            let executed_clone = executed.clone();

            // Act
            executor.spawn(async move {
                executed_clone.fetch_add(1, Ordering::SeqCst);
            });

            sleep(Duration::from_millis(100)).await;

            // Assert
            assert_eq!(executed.load(Ordering::SeqCst), 1);

            executor.shutdown().await;
        }

        #[tokio::test]
        async fn test_spawn_multiple_tasks() {
            // Arrange
            let executor = WorkStealingExecutor::new(4);
            let counter = Arc::new(AtomicUsize::new(0));
            let task_count = 100;

            // Act
            for _ in 0..task_count {
                let counter = counter.clone();
                executor.spawn(async move {
                    counter.fetch_add(1, Ordering::Relaxed);
                });
            }

            sleep(Duration::from_millis(200)).await;

            // Assert
            assert_eq!(counter.load(Ordering::Relaxed), task_count);

            executor.shutdown().await;
        }
    }

    mod executor_metrics {
        use super::*;

        #[tokio::test]
        async fn test_metrics_track_spawned_tasks() {
            // Arrange
            let executor = WorkStealingExecutor::new(2);

            // Act
            for _ in 0..50 {
                executor.spawn(async {
                    sleep(Duration::from_millis(1)).await;
                });
            }

            sleep(Duration::from_millis(10)).await;

            // Assert
            let metrics = executor.metrics();
            assert_eq!(metrics.tasks_spawned.load(Ordering::Relaxed), 50);

            executor.shutdown().await;
        }

        #[tokio::test]
        async fn test_metrics_track_completed_tasks() {
            // Arrange
            let executor = WorkStealingExecutor::new(4);
            let task_count = 20;

            // Act
            for _ in 0..task_count {
                executor.spawn(async {});
            }

            sleep(Duration::from_millis(100)).await;

            // Assert
            let metrics = executor.metrics();
            assert!(metrics.tasks_completed.load(Ordering::Relaxed) >= task_count);

            executor.shutdown().await;
        }
    }

    mod executor_workload {
        use super::*;

        #[tokio::test]
        async fn test_cpu_bound_workload() {
            // Arrange
            let executor = WorkStealingExecutor::new(4);
            let completed = Arc::new(AtomicUsize::new(0));

            // Act: Spawn CPU-bound tasks
            for i in 0..50 {
                let completed = completed.clone();
                executor.spawn(async move {
                    // Simulate CPU work
                    let mut sum = 0;
                    for j in 0..i * 100 {
                        sum += j;
                    }
                    std::hint::black_box(sum);
                    completed.fetch_add(1, Ordering::Relaxed);
                });
            }

            sleep(Duration::from_millis(200)).await;

            // Assert
            assert_eq!(completed.load(Ordering::Relaxed), 50);

            executor.shutdown().await;
        }

        #[tokio::test]
        async fn test_io_bound_workload() {
            // Arrange
            let executor = WorkStealingExecutor::new(4);
            let completed = Arc::new(AtomicUsize::new(0));

            // Act: Spawn I/O-bound tasks
            for _ in 0..20 {
                let completed = completed.clone();
                executor.spawn(async move {
                    sleep(Duration::from_millis(10)).await;
                    completed.fetch_add(1, Ordering::Relaxed);
                });
            }

            sleep(Duration::from_millis(300)).await;

            // Assert
            assert_eq!(completed.load(Ordering::Relaxed), 20);

            executor.shutdown().await;
        }

        #[tokio::test]
        async fn test_mixed_workload() {
            // Arrange
            let executor = WorkStealingExecutor::new(4);
            let cpu_done = Arc::new(AtomicUsize::new(0));
            let io_done = Arc::new(AtomicUsize::new(0));

            // Act: Spawn mixed workload
            for i in 0..40 {
                if i % 2 == 0 {
                    // CPU-bound
                    let cpu_done = cpu_done.clone();
                    executor.spawn(async move {
                        let mut sum = 0;
                        for j in 0..1000 {
                            sum += j;
                        }
                        std::hint::black_box(sum);
                        cpu_done.fetch_add(1, Ordering::Relaxed);
                    });
                } else {
                    // I/O-bound
                    let io_done = io_done.clone();
                    executor.spawn(async move {
                        sleep(Duration::from_millis(5)).await;
                        io_done.fetch_add(1, Ordering::Relaxed);
                    });
                }
            }

            sleep(Duration::from_millis(300)).await;

            // Assert
            assert_eq!(cpu_done.load(Ordering::Relaxed), 20);
            assert_eq!(io_done.load(Ordering::Relaxed), 20);

            executor.shutdown().await;
        }
    }

    mod executor_stress {
        use super::*;

        #[tokio::test]
        async fn test_many_tasks_stress() {
            // Arrange
            let executor = WorkStealingExecutor::new(8);
            let counter = Arc::new(AtomicUsize::new(0));
            let task_count = 1000;

            // Act
            let start = std::time::Instant::now();

            for _ in 0..task_count {
                let counter = counter.clone();
                executor.spawn(async move {
                    counter.fetch_add(1, Ordering::Relaxed);
                });
            }

            sleep(Duration::from_millis(500)).await;
            let elapsed = start.elapsed();

            // Assert
            assert_eq!(counter.load(Ordering::Relaxed), task_count);
            println!("Completed {} tasks in {:?}", task_count, elapsed);

            executor.shutdown().await;
        }

        #[tokio::test]
        async fn test_rapid_spawn_and_complete() {
            // Arrange
            let executor = WorkStealingExecutor::new(4);
            let completed = Arc::new(AtomicUsize::new(0));

            // Act: Rapidly spawn tasks
            for batch in 0..10 {
                for _ in 0..100 {
                    let completed = completed.clone();
                    executor.spawn(async move {
                        completed.fetch_add(1, Ordering::Relaxed);
                    });
                }
                sleep(Duration::from_millis(10)).await;
            }

            sleep(Duration::from_millis(200)).await;

            // Assert
            assert_eq!(completed.load(Ordering::Relaxed), 1000);

            executor.shutdown().await;
        }
    }

    mod executor_shutdown {
        use super::*;

        #[tokio::test]
        async fn test_graceful_shutdown() {
            // Arrange
            let executor = WorkStealingExecutor::new(4);
            let started = Arc::new(AtomicUsize::new(0));

            for _ in 0..10 {
                let started = started.clone();
                executor.spawn(async move {
                    started.fetch_add(1, Ordering::Relaxed);
                    sleep(Duration::from_millis(50)).await;
                });
            }

            sleep(Duration::from_millis(20)).await;

            // Act: Shutdown
            executor.shutdown().await;

            // Assert: Should complete cleanly
            assert!(started.load(Ordering::Relaxed) > 0);
        }
    }
}

#[cfg(not(feature = "async-v2"))]
mod work_stealing_executor_tests {
    #[test]
    fn test_feature_disabled() {
        // This test ensures the module compiles even without async-v2 feature
        assert!(true, "Work-stealing executor requires async-v2 feature");
    }
}
