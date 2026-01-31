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
/// # Object Safety
///
/// This trait is object-safe, meaning it can be used as a trait object
/// (`Box<dyn WorkUnit>`). This allows heterogeneous collections of different
/// work unit types to be executed by the same thread pool.
///
/// # Examples
///
/// Basic implementation:
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
///
/// Error handling example:
///
/// ```ignore
/// use crate::threading::WorkUnit;
///
/// struct ValidationTask {
///     value: i32,
/// }
///
/// impl WorkUnit for ValidationTask {
///     fn identifier(&self) -> String {
///         format!("validation_{}", self.value)
///     }
///
///     fn execute(&self) -> Result<(), String> {
///         if self.value < 0 {
///             Err(format!("Invalid value: {}", self.value))
///         } else {
///             Ok(())
///         }
///     }
/// }
/// ```
///
/// Using as trait objects:
///
/// ```ignore
/// use crate::threading::WorkUnit;
///
/// let tasks: Vec<Box<dyn WorkUnit>> = vec![
///     Box::new(MyTask { id: 1, data: "first".to_string() }),
///     Box::new(MyTask { id: 2, data: "second".to_string() }),
/// ];
/// ```
pub trait WorkUnit: Send + 'static {
    /// Returns a unique identifier for this work unit.
    ///
    /// This identifier is used for logging and error reporting. It should be
    /// descriptive enough to identify the specific work unit in logs.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use crate::threading::WorkUnit;
    /// # struct Task { id: usize }
    /// # impl WorkUnit for Task {
    /// fn identifier(&self) -> String {
    ///     format!("task_{}", self.id)
    /// }
    /// #     fn execute(&self) -> Result<(), String> { Ok(()) }
    /// # }
    /// ```
    fn identifier(&self) -> String;

    /// Executes the work unit.
    ///
    /// This method contains the actual logic to be executed in parallel.
    /// Implementations should handle errors gracefully and return descriptive
    /// error messages.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the work completed successfully
    /// * `Err(String)` with a descriptive error message if the work failed
    ///
    /// # Examples
    ///
    /// Successful execution:
    ///
    /// ```ignore
    /// # use crate::threading::WorkUnit;
    /// # struct Task;
    /// # impl WorkUnit for Task {
    /// #     fn identifier(&self) -> String { "task".to_string() }
    /// fn execute(&self) -> Result<(), String> {
    ///     // Do some work
    ///     Ok(())
    /// }
    /// # }
    /// ```
    ///
    /// Error handling:
    ///
    /// ```ignore
    /// # use crate::threading::WorkUnit;
    /// # struct Task { valid: bool }
    /// # impl WorkUnit for Task {
    /// #     fn identifier(&self) -> String { "task".to_string() }
    /// fn execute(&self) -> Result<(), String> {
    ///     if !self.valid {
    ///         return Err("Task validation failed".to_string());
    ///     }
    ///     // Do work
    ///     Ok(())
    /// }
    /// # }
    /// ```
    fn execute(&self) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test implementation: Simple task that always succeeds
    struct SuccessTask {
        id: usize,
        data: String,
    }

    impl WorkUnit for SuccessTask {
        fn identifier(&self) -> String {
            format!("success_task_{}", self.id)
        }

        fn execute(&self) -> Result<(), String> {
            Ok(())
        }
    }

    // Test implementation: Task that fails
    struct FailureTask {
        id: usize,
        error_message: String,
    }

    impl WorkUnit for FailureTask {
        fn identifier(&self) -> String {
            format!("failure_task_{}", self.id)
        }

        fn execute(&self) -> Result<(), String> {
            Err(self.error_message.clone())
        }
    }

    // Test implementation: Task with conditional logic
    struct ConditionalTask {
        id: usize,
        should_succeed: bool,
    }

    impl WorkUnit for ConditionalTask {
        fn identifier(&self) -> String {
            format!("conditional_task_{}", self.id)
        }

        fn execute(&self) -> Result<(), String> {
            if self.should_succeed {
                Ok(())
            } else {
                Err(format!("Task {} failed conditionally", self.id))
            }
        }
    }

    #[test]
    fn test_workunit_identifier() {
        let task = SuccessTask {
            id: 42,
            data: "test data".to_string(),
        };
        assert_eq!(task.identifier(), "success_task_42");
    }

    #[test]
    fn test_workunit_execute_success() {
        let task = SuccessTask {
            id: 1,
            data: "test".to_string(),
        };
        let result = task.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_workunit_execute_failure() {
        let task = FailureTask {
            id: 1,
            error_message: "Test error message".to_string(),
        };
        let result = task.execute();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Test error message");
    }

    #[test]
    fn test_workunit_conditional_success() {
        let task = ConditionalTask {
            id: 1,
            should_succeed: true,
        };
        assert!(task.execute().is_ok());
    }

    #[test]
    fn test_workunit_conditional_failure() {
        let task = ConditionalTask {
            id: 1,
            should_succeed: false,
        };
        let result = task.execute();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Task 1 failed"));
    }

    #[test]
    fn test_workunit_trait_object() {
        // Verify trait is object-safe by creating trait objects
        let tasks: Vec<Box<dyn WorkUnit>> = vec![
            Box::new(SuccessTask {
                id: 1,
                data: "first".to_string(),
            }),
            Box::new(SuccessTask {
                id: 2,
                data: "second".to_string(),
            }),
        ];

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].identifier(), "success_task_1");
        assert_eq!(tasks[1].identifier(), "success_task_2");
    }

    #[test]
    fn test_workunit_heterogeneous_trait_objects() {
        // Verify different types can be stored as trait objects
        let tasks: Vec<Box<dyn WorkUnit>> = vec![
            Box::new(SuccessTask {
                id: 1,
                data: "data".to_string(),
            }),
            Box::new(FailureTask {
                id: 2,
                error_message: "error".to_string(),
            }),
            Box::new(ConditionalTask {
                id: 3,
                should_succeed: true,
            }),
        ];

        assert_eq!(tasks.len(), 3);

        // Execute each and verify results
        let results: Vec<Result<(), String>> = tasks.iter().map(|t| t.execute()).collect();

        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
    }

    #[test]
    fn test_workunit_send_bound() {
        // This test verifies that WorkUnit has Send bound
        // If WorkUnit didn't have Send, this wouldn't compile
        fn assert_send<T: Send>() {}
        assert_send::<Box<dyn WorkUnit>>();
    }

    #[test]
    fn test_workunit_static_lifetime() {
        // Verify that WorkUnit requires 'static lifetime
        let task = SuccessTask {
            id: 1,
            data: "owned data".to_string(),
        };

        // This works because task owns its data
        let _boxed: Box<dyn WorkUnit> = Box::new(task);
    }
}
