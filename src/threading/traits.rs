//! Traits for the threading module.
//!
//! This module defines the core trait for units of work that can be executed
//! in parallel by the thread pool.

/// A unit of work that can be executed in parallel.
///
/// Types implementing this trait can be submitted to the `ThreadPool` for
/// parallel execution. Each work unit must be thread-safe (`Send`) and have
/// a static lifetime.
///
/// # Examples
///
/// ```ignore
/// use crate::threading::WorkUnit;
///
/// struct MyTask {
///     id: usize,
///     data: String,
/// }
///
/// impl WorkUnit for MyTask {
///     fn identifier(&self) -> String {
///         format!("task_{}", self.id)
///     }
///
///     fn execute(&self) -> Result<(), String> {
///         // Perform the work
///         println!("Processing: {}", self.data);
///         Ok(())
///     }
/// }
/// ```
pub trait WorkUnit: Send + 'static {
    /// Returns a unique identifier for this work unit.
    ///
    /// This identifier is used for logging and error reporting. It should be
    /// descriptive enough to identify the specific work unit in logs.
    fn identifier(&self) -> String;

    /// Executes the work unit.
    ///
    /// This method contains the actual logic to be executed in parallel.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the work completed successfully
    /// * `Err(String)` with a descriptive error message if the work failed
    fn execute(&self) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test implementation for simple success case
    struct SimpleTask {
        id: usize,
        name: String,
    }

    impl WorkUnit for SimpleTask {
        fn identifier(&self) -> String {
            format!("simple_{}_{}", self.id, self.name)
        }

        fn execute(&self) -> Result<(), String> {
            Ok(())
        }
    }

    // Test implementation for failure case
    struct FailingTask {
        id: usize,
        error_msg: String,
    }

    impl WorkUnit for FailingTask {
        fn identifier(&self) -> String {
            format!("failing_{}", self.id)
        }

        fn execute(&self) -> Result<(), String> {
            Err(self.error_msg.clone())
        }
    }

    // Test implementation with mutable internal state
    struct StatefulTask {
        id: usize,
        counter: std::sync::Arc<std::sync::Mutex<usize>>,
    }

    impl WorkUnit for StatefulTask {
        fn identifier(&self) -> String {
            format!("stateful_{}", self.id)
        }

        fn execute(&self) -> Result<(), String> {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;
            Ok(())
        }
    }

    #[test]
    fn test_workunit_simple_implementation() {
        let task = SimpleTask {
            id: 1,
            name: "test".to_string(),
        };
        assert_eq!(task.identifier(), "simple_1_test");
        assert!(task.execute().is_ok());
    }

    #[test]
    fn test_workunit_failing_implementation() {
        let task = FailingTask {
            id: 42,
            error_msg: "Test error".to_string(),
        };
        assert_eq!(task.identifier(), "failing_42");
        let result = task.execute();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Test error");
    }

    #[test]
    fn test_workunit_stateful_implementation() {
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
        let task = StatefulTask {
            id: 1,
            counter: counter.clone(),
        };
        assert_eq!(task.identifier(), "stateful_1");
        assert!(task.execute().is_ok());
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_workunit_trait_object() {
        // Test that WorkUnit is object-safe by creating trait objects
        let task1: Box<dyn WorkUnit> = Box::new(SimpleTask {
            id: 1,
            name: "boxed".to_string(),
        });
        let task2: Box<dyn WorkUnit> = Box::new(FailingTask {
            id: 2,
            error_msg: "boxed error".to_string(),
        });

        assert_eq!(task1.identifier(), "simple_1_boxed");
        assert!(task1.execute().is_ok());

        assert_eq!(task2.identifier(), "failing_2");
        assert!(task2.execute().is_err());
    }

    #[test]
    fn test_workunit_vec_of_trait_objects() {
        // Test that we can create a collection of different WorkUnit implementations
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
        let tasks: Vec<Box<dyn WorkUnit>> = vec![
            Box::new(SimpleTask {
                id: 1,
                name: "first".to_string(),
            }),
            Box::new(SimpleTask {
                id: 2,
                name: "second".to_string(),
            }),
            Box::new(StatefulTask {
                id: 3,
                counter: counter.clone(),
            }),
        ];

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].identifier(), "simple_1_first");
        assert_eq!(tasks[1].identifier(), "simple_2_second");
        assert_eq!(tasks[2].identifier(), "stateful_3");

        // Execute all tasks
        for task in tasks {
            let _ = task.execute();
        }
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_workunit_send_trait_bound() {
        // This test verifies that WorkUnit requires Send
        // If this compiles, the Send bound is working correctly
        fn require_send<T: Send>(_: T) {}

        let task = SimpleTask {
            id: 1,
            name: "send_test".to_string(),
        };
        require_send(task);
    }

    #[test]
    fn test_workunit_identifier_uniqueness() {
        let task1 = SimpleTask {
            id: 1,
            name: "task".to_string(),
        };
        let task2 = SimpleTask {
            id: 2,
            name: "task".to_string(),
        };
        let task3 = SimpleTask {
            id: 1,
            name: "other".to_string(),
        };

        // Identifiers should be different for different tasks
        assert_ne!(task1.identifier(), task2.identifier());
        assert_ne!(task1.identifier(), task3.identifier());
        assert_ne!(task2.identifier(), task3.identifier());
    }

    #[test]
    fn test_workunit_execute_multiple_times() {
        // Verify that execute can be called multiple times on the same task
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
        let task = StatefulTask {
            id: 1,
            counter: counter.clone(),
        };

        assert!(task.execute().is_ok());
        assert_eq!(*counter.lock().unwrap(), 1);

        assert!(task.execute().is_ok());
        assert_eq!(*counter.lock().unwrap(), 2);

        assert!(task.execute().is_ok());
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    #[test]
    fn test_workunit_error_messages() {
        let error_messages = [
            "Simple error",
            "Error with special chars: !@#$%^&*()",
            "Multi\nline\nerror",
            "",
        ];

        for (i, msg) in error_messages.iter().enumerate() {
            let task = FailingTask {
                id: i,
                error_msg: msg.to_string(),
            };
            match task.execute() {
                Ok(_) => panic!("Expected error"),
                Err(e) => assert_eq!(e, *msg),
            }
        }
    }
}
