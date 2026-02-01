//! Thread pool implementation for parallel work execution.
//!
//! This module provides a thread pool that can execute multiple work units
//! in parallel using a configurable number of worker threads.

use crate::threading::{AggregatedError, Config, ErrorCollector, ExecutionError, WorkUnit};
use log::{debug, error};

/// A thread pool for executing work units in parallel.
///
/// The thread pool spawns a fixed number of worker threads and distributes
/// work units among them. The skeleton implementation currently collects all
/// errors; a future implementation may add fail-fast behavior where the first
/// error stops remaining work.
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
    /// for all work to complete. The current skeleton implementation collects
    /// all errors and continues execution; a future implementation may add
    /// fail-fast behavior to stop on the first error.
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

        // For now, this is a skeleton implementation
        // The full implementation will include:
        // 1. Creating worker threads
        // 2. Distributing work via channels
        // 3. Collecting results using ErrorCollector for thread-safe error aggregation
        // 4. Fail-fast error handling
        // 5. Proper thread cleanup
        //
        // Example for future parallel implementation:
        // ```
        // let error_collector = ErrorCollector::new();
        // let handles: Vec<_> = work_units.into_iter().map(|unit| {
        //     let collector = error_collector.clone();
        //     thread::spawn(move || {
        //         match unit.execute() {
        //             Ok(()) => {},
        //             Err(e) => collector.add(ExecutionError::new(unit.identifier(), e)),
        //         }
        //     })
        // }).collect();
        // for handle in handles {
        //     handle.join().unwrap();
        // }
        // error_collector.into_result()
        // ```

        // Skeleton: Execute sequentially for now
        let mut errors = Vec::new();

        for unit in work_units {
            let identifier = unit.identifier();
            debug!("Executing work unit: {}", identifier);

            match unit.execute() {
                Ok(()) => {
                    debug!("Work unit {} completed successfully", identifier);
                }
                Err(e) => {
                    error!("Work unit {} failed: {}", identifier, e);
                    errors.push(ExecutionError::new(identifier, e));
                }
            }
        }

        if errors.is_empty() {
            debug!("All {} work units completed successfully", work_count);
            Ok(())
        } else {
            Err(AggregatedError::new(errors))
        }
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
}
