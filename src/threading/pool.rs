//! Thread pool implementation for parallel work execution.
//!
//! This module provides a thread pool that can execute multiple work units
//! in parallel using a configurable number of worker threads.

use crate::threading::{AggregatedError, Config, ErrorCollector, ExecutionError, WorkUnit};
use log::{debug, error};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// A thread pool for executing work units in parallel.
///
/// The thread pool spawns a fixed number of worker threads and distributes
/// work units among them using channels. It collects all errors and continues
/// execution until all work units have been processed. Worker threads are
/// automatically joined and cleaned up after all work completes.
///
/// # Implementation Details
///
/// - Uses `std::sync::mpsc` channels for work distribution
/// - Receiver is wrapped in `Arc<Mutex<>>` for sharing among workers
/// - Thread count is limited to the minimum of configured threads and work count
/// - Errors are collected using thread-safe `ErrorCollector`
/// - Worker thread panics are caught and reported as errors
///
/// # Examples
///
/// ```ignore
/// use crate::threading::{Config, ThreadPool, WorkUnit};
///
/// struct MyTask { id: usize }
///
/// impl WorkUnit for MyTask {
///     fn identifier(&self) -> String {
///         format!("task_{}", self.id)
///     }
///     
///     fn execute(&self) -> Result<(), String> {
///         println!("Executing task {}", self.id);
///         Ok(())
///     }
/// }
///
/// let config = Config::new(4);
/// let pool = ThreadPool::new(config);
/// let tasks: Vec<Box<dyn WorkUnit>> = vec![
///     Box::new(MyTask { id: 1 }),
///     Box::new(MyTask { id: 2 }),
/// ];
/// pool.execute(tasks).unwrap();
/// ```
pub struct ThreadPool {
    config: Config,
}

impl ThreadPool {
    /// Creates a new thread pool with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration specifying the number of threads
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::threading::{Config, ThreadPool};
    ///
    /// let config = Config::new(4);
    /// let pool = ThreadPool::new(config);
    /// ```
    pub fn new(config: Config) -> Self {
        debug!(
            "Creating thread pool with {} threads",
            config.thread_count()
        );
        Self { config }
    }

    /// Executes a collection of work units in parallel.
    ///
    /// This method distributes the work units among worker threads and waits
    /// for all work to complete. It collects all errors and continues execution
    /// until all work units have been processed.
    ///
    /// # Arguments
    ///
    /// * `work_units` - Vector of boxed work units to execute
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all work units completed successfully
    /// * `Err(AggregatedError)` if one or more work units failed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::threading::{Config, ThreadPool};
    ///
    /// let pool = ThreadPool::new(Config::default());
    /// let tasks: Vec<Box<dyn WorkUnit>> = create_tasks();
    /// match pool.execute(tasks) {
    ///     Ok(()) => println!("All tasks completed successfully"),
    ///     Err(e) => eprintln!("Execution failed: {}", e),
    /// }
    /// ```
    pub fn execute(&self, work_units: Vec<Box<dyn WorkUnit>>) -> Result<(), AggregatedError> {
        let work_count = work_units.len();
        debug!("Executing {} work units", work_count);

        // If there are no work units, return success immediately
        if work_units.is_empty() {
            return Ok(());
        }

        // Create a channel for distributing work to threads
        let (sender, receiver) = mpsc::channel::<Box<dyn WorkUnit>>();

        // Wrap receiver in Arc<Mutex<>> so multiple threads can share it
        let receiver = Arc::new(Mutex::new(receiver));

        // Create error collector for thread-safe error aggregation
        let error_collector = ErrorCollector::new();

        // Spawn worker threads
        let thread_count = self.config.thread_count().min(work_count);
        debug!("Spawning {} worker threads", thread_count);

        let mut handles = Vec::with_capacity(thread_count);

        for worker_id in 0..thread_count {
            let receiver = Arc::clone(&receiver);
            let collector = error_collector.clone();

            let handle = thread::spawn(move || {
                debug!("Worker {} started", worker_id);

                // Process work units from the channel until it's closed
                loop {
                    // Lock the receiver to get the next work unit
                    let work_unit = {
                        let rx = receiver.lock().unwrap();
                        rx.recv()
                    };

                    match work_unit {
                        Ok(unit) => {
                            let identifier = unit.identifier();
                            debug!("Worker {} executing work unit: {}", worker_id, identifier);

                            match unit.execute() {
                                Ok(()) => {
                                    debug!(
                                        "Worker {} completed work unit: {}",
                                        worker_id, identifier
                                    );
                                }
                                Err(e) => {
                                    error!(
                                        "Worker {} failed work unit {}: {}",
                                        worker_id, identifier, e
                                    );
                                    collector.add(ExecutionError::new(identifier, e));
                                }
                            }
                        }
                        Err(_) => {
                            // Channel closed, no more work
                            debug!("Worker {} received channel close signal", worker_id);
                            break;
                        }
                    }
                }

                debug!("Worker {} finished", worker_id);
            });

            handles.push(handle);
        }

        // Send all work units to the channel
        for unit in work_units {
            // If send fails, it means all receivers have been dropped (shouldn't happen)
            if sender.send(unit).is_err() {
                error!("Failed to send work unit to channel - all workers have terminated");
                break;
            }
        }

        // Drop the sender to signal workers that no more work is coming
        drop(sender);

        // Wait for all worker threads to complete
        for (worker_id, handle) in handles.into_iter().enumerate() {
            match handle.join() {
                Ok(()) => {
                    debug!("Worker {} joined successfully", worker_id);
                }
                Err(e) => {
                    error!("Worker {} panicked: {:?}", worker_id, e);
                    // Add a generic error for worker panic
                    error_collector.add(ExecutionError::new(
                        format!("worker_{}", worker_id),
                        format!("Worker thread panicked: {:?}", e),
                    ));
                }
            }
        }

        // Convert error collector to result
        error_collector.into_result().map(|_| {
            debug!("All {} work units completed successfully", work_count);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTask {
        id: usize,
        should_fail: bool,
    }

    impl WorkUnit for TestTask {
        fn identifier(&self) -> String {
            format!("test_task_{}", self.id)
        }

        fn execute(&self) -> Result<(), String> {
            if self.should_fail {
                Err(format!("Task {} failed", self.id))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_thread_pool_new() {
        let config = Config::new(4);
        let pool = ThreadPool::new(config);
        assert_eq!(pool.config.thread_count(), 4);
    }

    #[test]
    fn test_execute_empty() {
        let pool = ThreadPool::new(Config::new(4));
        let result = pool.execute(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_success() {
        let pool = ThreadPool::new(Config::new(4));
        let tasks: Vec<Box<dyn WorkUnit>> = vec![Box::new(TestTask {
            id: 1,
            should_fail: false,
        })];
        let result = pool.execute(tasks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_single_failure() {
        let pool = ThreadPool::new(Config::new(4));
        let tasks: Vec<Box<dyn WorkUnit>> = vec![Box::new(TestTask {
            id: 1,
            should_fail: true,
        })];
        let result = pool.execute(tasks);
        assert!(result.is_err());

        if let Err(agg) = result {
            assert_eq!(agg.len(), 1);
            assert_eq!(agg.first().unit_identifier, "test_task_1");
        }
    }

    #[test]
    fn test_execute_multiple_success() {
        let pool = ThreadPool::new(Config::new(4));
        let tasks: Vec<Box<dyn WorkUnit>> = vec![
            Box::new(TestTask {
                id: 1,
                should_fail: false,
            }),
            Box::new(TestTask {
                id: 2,
                should_fail: false,
            }),
            Box::new(TestTask {
                id: 3,
                should_fail: false,
            }),
        ];
        let result = pool.execute(tasks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_multiple_failures() {
        let pool = ThreadPool::new(Config::new(4));
        let tasks: Vec<Box<dyn WorkUnit>> = vec![
            Box::new(TestTask {
                id: 1,
                should_fail: true,
            }),
            Box::new(TestTask {
                id: 2,
                should_fail: false,
            }),
            Box::new(TestTask {
                id: 3,
                should_fail: true,
            }),
        ];
        let result = pool.execute(tasks);
        assert!(result.is_err());

        if let Err(agg) = result {
            assert_eq!(agg.len(), 2);
            assert!(agg
                .errors()
                .iter()
                .any(|e| e.unit_identifier == "test_task_1"));
            assert!(agg
                .errors()
                .iter()
                .any(|e| e.unit_identifier == "test_task_3"));
        }
    }

    #[test]
    fn test_parallel_execution() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::time::Duration;

        struct SlowTask {
            id: usize,
            counter: Arc<Mutex<Vec<usize>>>,
        }

        impl WorkUnit for SlowTask {
            fn identifier(&self) -> String {
                format!("slow_task_{}", self.id)
            }

            fn execute(&self) -> Result<(), String> {
                // Simulate work with a small delay
                thread::sleep(Duration::from_millis(10));
                let mut counter = self.counter.lock().unwrap();
                counter.push(self.id);
                Ok(())
            }
        }

        let pool = ThreadPool::new(Config::new(4));
        let counter = Arc::new(Mutex::new(Vec::new()));
        let task_count = 20;

        let tasks: Vec<Box<dyn WorkUnit>> = (0..task_count)
            .map(|i| {
                Box::new(SlowTask {
                    id: i,
                    counter: Arc::clone(&counter),
                }) as Box<dyn WorkUnit>
            })
            .collect();

        let start = std::time::Instant::now();
        let result = pool.execute(tasks);
        let duration = start.elapsed();

        assert!(result.is_ok());

        // Verify all tasks completed
        let completed = counter.lock().unwrap();
        assert_eq!(completed.len(), task_count);

        // With 4 threads and 20 tasks of 10ms each, parallel execution should take
        // roughly 50-60ms (5 batches), while sequential would take ~200ms
        // Allow some margin for thread overhead
        assert!(
            duration.as_millis() < 150,
            "Execution took {}ms, expected less than 150ms for parallel execution",
            duration.as_millis()
        );
    }

    #[test]
    fn test_single_thread_pool() {
        let pool = ThreadPool::new(Config::new(1));
        let tasks: Vec<Box<dyn WorkUnit>> = vec![
            Box::new(TestTask {
                id: 1,
                should_fail: false,
            }),
            Box::new(TestTask {
                id: 2,
                should_fail: false,
            }),
            Box::new(TestTask {
                id: 3,
                should_fail: false,
            }),
        ];
        let result = pool.execute(tasks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_more_threads_than_work() {
        let pool = ThreadPool::new(Config::new(10));
        let tasks: Vec<Box<dyn WorkUnit>> = vec![
            Box::new(TestTask {
                id: 1,
                should_fail: false,
            }),
            Box::new(TestTask {
                id: 2,
                should_fail: false,
            }),
        ];
        let result = pool.execute(tasks);
        assert!(result.is_ok());
    }
}
