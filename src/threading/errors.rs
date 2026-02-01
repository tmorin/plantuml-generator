//! Error types for the threading module.
//!
//! This module defines error types for thread pool execution, including
//! error aggregation for collecting failures from multiple work units.

use std::fmt;
use std::sync::{Arc, Mutex};

/// An error that occurred during execution of a work unit.
///
/// This struct captures both the identifier of the failed work unit and
/// the error message.
#[derive(Debug, Clone)]
pub struct ExecutionError {
    /// Identifier of the work unit that failed.
    pub unit_identifier: String,
    /// Error message describing the failure.
    pub message: String,
}

impl ExecutionError {
    /// Creates a new execution error.
    ///
    /// # Arguments
    ///
    /// * `unit_identifier` - Identifier of the failed work unit
    /// * `message` - Error message describing the failure
    pub fn new(unit_identifier: String, message: String) -> Self {
        Self {
            unit_identifier,
            message,
        }
    }
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.unit_identifier, self.message)
    }
}

/// An aggregated error containing multiple execution errors.
///
/// This error type is used when multiple work units fail during parallel
/// execution. It collects all individual errors and provides a combined
/// error message.
#[derive(Debug)]
pub struct AggregatedError {
    /// Collection of individual execution errors.
    errors: Vec<ExecutionError>,
}

impl AggregatedError {
    /// Creates a new aggregated error from a vector of execution errors.
    ///
    /// # Arguments
    ///
    /// * `errors` - Vector of execution errors
    ///
    /// # Panics
    ///
    /// Panics if the errors vector is empty.
    pub fn new(errors: Vec<ExecutionError>) -> Self {
        assert!(!errors.is_empty(), "AggregatedError cannot be empty");
        Self { errors }
    }

    /// Returns a reference to the first error.
    ///
    /// This is useful for fail-fast scenarios where you want to inspect
    /// the first error that occurred.
    pub fn first(&self) -> &ExecutionError {
        &self.errors[0]
    }

    /// Returns a reference to all errors.
    pub fn errors(&self) -> &[ExecutionError] {
        &self.errors
    }

    /// Returns the number of errors.
    ///
    /// This will always be at least 1 since AggregatedError cannot be
    /// constructed with an empty error vector.
    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl fmt::Display for AggregatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.errors.len() == 1 {
            write!(f, "Execution failed: {}", self.errors[0])
        } else {
            writeln!(f, "Execution failed with {} errors:", self.errors.len())?;
            for (i, error) in self.errors.iter().enumerate() {
                writeln!(f, "  {}. {}", i + 1, error)?;
            }
            Ok(())
        }
    }
}

impl std::error::Error for AggregatedError {}

/// A thread-safe collector for execution errors.
///
/// This struct provides a thread-safe way to collect errors from multiple
/// worker threads during parallel execution. It uses `Arc<Mutex<Vec<ExecutionError>>>`
/// internally to ensure safe concurrent access.
///
/// # Examples
///
/// ```ignore
/// use crate::threading::errors::{ErrorCollector, ExecutionError};
/// use std::thread;
///
/// let collector = ErrorCollector::new();
///
/// // Clone the collector for use in multiple threads
/// let collector_clone = collector.clone();
/// let handle = thread::spawn(move || {
///     collector_clone.add(ExecutionError::new(
///         "task_1".to_string(),
///         "Failed to process".to_string(),
///     ));
/// });
///
/// handle.join().unwrap();
///
/// // Check if any errors occurred
/// if collector.has_errors() {
///     let aggregated = collector.into_result().unwrap_err();
///     println!("Errors occurred: {}", aggregated);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ErrorCollector {
    errors: Arc<Mutex<Vec<ExecutionError>>>,
}

impl ErrorCollector {
    /// Creates a new empty error collector.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::threading::errors::ErrorCollector;
    ///
    /// let collector = ErrorCollector::new();
    /// assert!(!collector.has_errors());
    /// ```
    pub fn new() -> Self {
        Self {
            errors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds an error to the collection.
    ///
    /// This method is thread-safe and can be called from multiple threads
    /// concurrently.
    ///
    /// # Arguments
    ///
    /// * `error` - The execution error to add
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::threading::errors::{ErrorCollector, ExecutionError};
    ///
    /// let collector = ErrorCollector::new();
    /// collector.add(ExecutionError::new(
    ///     "task_1".to_string(),
    ///     "Failed".to_string(),
    /// ));
    /// assert!(collector.has_errors());
    /// ```
    pub fn add(&self, error: ExecutionError) {
        let mut errors = self.errors.lock().unwrap();
        errors.push(error);
    }

    /// Checks if any errors have been collected.
    ///
    /// # Returns
    ///
    /// `true` if at least one error has been added, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::threading::errors::ErrorCollector;
    ///
    /// let collector = ErrorCollector::new();
    /// assert!(!collector.has_errors());
    /// ```
    pub fn has_errors(&self) -> bool {
        let errors = self.errors.lock().unwrap();
        !errors.is_empty()
    }

    /// Returns the number of errors collected.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::threading::errors::{ErrorCollector, ExecutionError};
    ///
    /// let collector = ErrorCollector::new();
    /// collector.add(ExecutionError::new("task_1".to_string(), "Error".to_string()));
    /// assert_eq!(collector.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        let errors = self.errors.lock().unwrap();
        errors.len()
    }

    /// Checks if the collector is empty.
    ///
    /// # Returns
    ///
    /// `true` if no errors have been collected, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        let errors = self.errors.lock().unwrap();
        errors.is_empty()
    }

    /// Consumes the collector and returns a Result.
    ///
    /// If no errors were collected, returns `Ok(())`. If errors were collected,
    /// returns `Err(AggregatedError)`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::threading::errors::{ErrorCollector, ExecutionError};
    ///
    /// let collector = ErrorCollector::new();
    /// assert!(collector.into_result().is_ok());
    ///
    /// let collector = ErrorCollector::new();
    /// collector.add(ExecutionError::new("task_1".to_string(), "Error".to_string()));
    /// assert!(collector.into_result().is_err());
    /// ```
    pub fn into_result(self) -> Result<(), AggregatedError> {
        let errors = Arc::try_unwrap(self.errors)
            .expect("Cannot convert ErrorCollector to Result while clones exist. Ensure all clones are dropped before calling into_result()")
            .into_inner()
            .unwrap();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AggregatedError::new(errors))
        }
    }

    /// Returns a snapshot of the current errors without consuming the collector.
    ///
    /// This method clones all errors and returns them in a Vec. The collector
    /// remains usable after this call.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::threading::errors::{ErrorCollector, ExecutionError};
    ///
    /// let collector = ErrorCollector::new();
    /// collector.add(ExecutionError::new("task_1".to_string(), "Error".to_string()));
    /// let errors = collector.snapshot();
    /// assert_eq!(errors.len(), 1);
    /// assert!(collector.has_errors()); // Collector is still usable
    /// ```
    pub fn snapshot(&self) -> Vec<ExecutionError> {
        let errors = self.errors.lock().unwrap();
        errors.clone()
    }
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_error_new() {
        let error = ExecutionError::new("task_1".to_string(), "Failed to process".to_string());
        assert_eq!(error.unit_identifier, "task_1");
        assert_eq!(error.message, "Failed to process");
    }

    #[test]
    fn test_execution_error_display() {
        let error = ExecutionError::new("task_1".to_string(), "Failed to process".to_string());
        let display = format!("{}", error);
        assert_eq!(display, "[task_1] Failed to process");
    }

    #[test]
    fn test_aggregated_error_single() {
        let error = ExecutionError::new("task_1".to_string(), "Failed".to_string());
        let agg = AggregatedError::new(vec![error]);

        assert_eq!(agg.len(), 1);
        assert_eq!(agg.first().unit_identifier, "task_1");
    }

    #[test]
    fn test_aggregated_error_multiple() {
        let errors = vec![
            ExecutionError::new("task_1".to_string(), "Error 1".to_string()),
            ExecutionError::new("task_2".to_string(), "Error 2".to_string()),
            ExecutionError::new("task_3".to_string(), "Error 3".to_string()),
        ];
        let agg = AggregatedError::new(errors);

        assert_eq!(agg.len(), 3);
        assert_eq!(agg.first().unit_identifier, "task_1");
        assert_eq!(agg.errors().len(), 3);
    }

    #[test]
    fn test_aggregated_error_display_single() {
        let error = ExecutionError::new("task_1".to_string(), "Failed".to_string());
        let agg = AggregatedError::new(vec![error]);
        let display = format!("{}", agg);
        assert_eq!(display, "Execution failed: [task_1] Failed");
    }

    #[test]
    fn test_aggregated_error_display_multiple() {
        let errors = vec![
            ExecutionError::new("task_1".to_string(), "Error 1".to_string()),
            ExecutionError::new("task_2".to_string(), "Error 2".to_string()),
        ];
        let agg = AggregatedError::new(errors);
        let display = format!("{}", agg);
        assert!(display.contains("Execution failed with 2 errors:"));
        assert!(display.contains("[task_1] Error 1"));
        assert!(display.contains("[task_2] Error 2"));
    }

    #[test]
    #[should_panic(expected = "AggregatedError cannot be empty")]
    fn test_aggregated_error_empty_panics() {
        AggregatedError::new(vec![]);
    }

    // ErrorCollector tests
    #[test]
    fn test_error_collector_new() {
        let collector = ErrorCollector::new();
        assert!(!collector.has_errors());
        assert!(collector.is_empty());
        assert_eq!(collector.len(), 0);
    }

    #[test]
    fn test_error_collector_add() {
        let collector = ErrorCollector::new();
        collector.add(ExecutionError::new(
            "task_1".to_string(),
            "Error 1".to_string(),
        ));
        assert!(collector.has_errors());
        assert!(!collector.is_empty());
        assert_eq!(collector.len(), 1);
    }

    #[test]
    fn test_error_collector_add_multiple() {
        let collector = ErrorCollector::new();
        collector.add(ExecutionError::new(
            "task_1".to_string(),
            "Error 1".to_string(),
        ));
        collector.add(ExecutionError::new(
            "task_2".to_string(),
            "Error 2".to_string(),
        ));
        collector.add(ExecutionError::new(
            "task_3".to_string(),
            "Error 3".to_string(),
        ));
        assert_eq!(collector.len(), 3);
    }

    #[test]
    fn test_error_collector_into_result_success() {
        let collector = ErrorCollector::new();
        let result = collector.into_result();
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_collector_into_result_failure() {
        let collector = ErrorCollector::new();
        collector.add(ExecutionError::new(
            "task_1".to_string(),
            "Error 1".to_string(),
        ));
        let result = collector.into_result();
        assert!(result.is_err());

        if let Err(agg) = result {
            assert_eq!(agg.len(), 1);
            assert_eq!(agg.first().unit_identifier, "task_1");
        }
    }

    #[test]
    fn test_error_collector_snapshot() {
        let collector = ErrorCollector::new();
        collector.add(ExecutionError::new(
            "task_1".to_string(),
            "Error 1".to_string(),
        ));
        collector.add(ExecutionError::new(
            "task_2".to_string(),
            "Error 2".to_string(),
        ));

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].unit_identifier, "task_1");
        assert_eq!(snapshot[1].unit_identifier, "task_2");

        // Collector should still be usable
        assert!(collector.has_errors());
        assert_eq!(collector.len(), 2);
    }

    #[test]
    fn test_error_collector_clone() {
        let collector = ErrorCollector::new();
        collector.add(ExecutionError::new(
            "task_1".to_string(),
            "Error 1".to_string(),
        ));

        let clone = collector.clone();
        assert_eq!(clone.len(), 1);

        // Adding to clone should also add to original (shared Arc)
        clone.add(ExecutionError::new(
            "task_2".to_string(),
            "Error 2".to_string(),
        ));
        assert_eq!(collector.len(), 2);
        assert_eq!(clone.len(), 2);
    }

    #[test]
    fn test_error_collector_thread_safety() {
        use std::thread;

        let collector = ErrorCollector::new();

        let mut handles = vec![];
        for i in 0..10 {
            let collector_clone = collector.clone();
            let handle = thread::spawn(move || {
                collector_clone.add(ExecutionError::new(
                    format!("task_{}", i),
                    format!("Error {}", i),
                ));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(collector.len(), 10);
        let snapshot = collector.snapshot();
        assert_eq!(snapshot.len(), 10);
    }

    #[test]
    fn test_error_collector_default() {
        let collector = ErrorCollector::default();
        assert!(!collector.has_errors());
        assert_eq!(collector.len(), 0);
    }

    #[test]
    fn test_error_collector_stress_high_concurrency() {
        use std::thread;

        let collector = ErrorCollector::new();
        let thread_count = 50;
        let errors_per_thread = 100;

        let mut handles = vec![];
        for thread_id in 0..thread_count {
            let collector_clone = collector.clone();
            let handle = thread::spawn(move || {
                for i in 0..errors_per_thread {
                    collector_clone.add(ExecutionError::new(
                        format!("task_{}_{}", thread_id, i),
                        format!("Error from thread {} iteration {}", thread_id, i),
                    ));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(collector.len(), thread_count * errors_per_thread);
        assert!(collector.has_errors());
    }

    #[test]
    fn test_error_collector_snapshot_consistency() {
        let collector = ErrorCollector::new();

        collector.add(ExecutionError::new(
            "task_1".to_string(),
            "Error 1".to_string(),
        ));
        let snapshot1 = collector.snapshot();

        collector.add(ExecutionError::new(
            "task_2".to_string(),
            "Error 2".to_string(),
        ));
        let snapshot2 = collector.snapshot();

        assert_eq!(snapshot1.len(), 1);
        assert_eq!(snapshot2.len(), 2);
        assert_eq!(collector.len(), 2);
    }

    #[test]
    fn test_aggregated_error_is_error_trait() {
        use std::error::Error;

        let error = ExecutionError::new("task_1".to_string(), "Test".to_string());
        let agg = AggregatedError::new(vec![error]);

        // Verify it implements Error trait
        let _: &dyn Error = &agg;
    }

    #[test]
    fn test_execution_error_clone() {
        let error1 = ExecutionError::new("task_1".to_string(), "Error".to_string());
        let error2 = error1.clone();

        assert_eq!(error1.unit_identifier, error2.unit_identifier);
        assert_eq!(error1.message, error2.message);
    }

    #[test]
    fn test_aggregated_error_errors_accessor() {
        let errors = vec![
            ExecutionError::new("task_1".to_string(), "Error 1".to_string()),
            ExecutionError::new("task_2".to_string(), "Error 2".to_string()),
        ];
        let agg = AggregatedError::new(errors);

        let accessed_errors = agg.errors();
        assert_eq!(accessed_errors.len(), 2);
        assert_eq!(accessed_errors[0].unit_identifier, "task_1");
        assert_eq!(accessed_errors[1].unit_identifier, "task_2");
    }

    #[test]
    fn test_error_collector_multiple_snapshots() {
        let collector = ErrorCollector::new();

        collector.add(ExecutionError::new(
            "task_1".to_string(),
            "Error 1".to_string(),
        ));
        let snapshot1 = collector.snapshot();
        let snapshot2 = collector.snapshot();

        // Both snapshots should be identical
        assert_eq!(snapshot1.len(), snapshot2.len());
        assert_eq!(snapshot1[0].unit_identifier, snapshot2[0].unit_identifier);
    }

    #[test]
    fn test_error_collector_concurrent_snapshots() {
        use std::thread;

        let collector = ErrorCollector::new();

        // Add some initial errors
        for i in 0..10 {
            collector.add(ExecutionError::new(
                format!("task_{}", i),
                format!("Error {}", i),
            ));
        }

        // Take snapshots concurrently
        let mut handles = vec![];
        for _ in 0..5 {
            let collector_clone = collector.clone();
            let handle = thread::spawn(move || {
                let snapshot = collector_clone.snapshot();
                assert_eq!(snapshot.len(), 10);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_aggregated_error_display_formatting() {
        let error1 = ExecutionError::new("task_1".to_string(), "First error".to_string());
        let error2 = ExecutionError::new("task_2".to_string(), "Second error".to_string());
        let error3 = ExecutionError::new("task_3".to_string(), "Third error".to_string());

        let agg = AggregatedError::new(vec![error1, error2, error3]);
        let display = format!("{}", agg);

        // Should contain the count
        assert!(display.contains("3 errors"));
        // Should contain all error messages
        assert!(display.contains("First error"));
        assert!(display.contains("Second error"));
        assert!(display.contains("Third error"));
        // Should contain enumeration
        assert!(display.contains("1."));
        assert!(display.contains("2."));
        assert!(display.contains("3."));
    }

    #[test]
    fn test_error_collector_empty_snapshot() {
        let collector = ErrorCollector::new();
        let snapshot = collector.snapshot();
        assert_eq!(snapshot.len(), 0);
        assert!(snapshot.is_empty());
    }

    #[test]
    fn test_execution_error_with_special_characters() {
        let error = ExecutionError::new(
            "task_with_unicode_🚀".to_string(),
            "Error with special chars: \n\t\"quotes\" and \\backslash\\".to_string(),
        );
        let display = format!("{}", error);
        assert!(display.contains("task_with_unicode_🚀"));
        assert!(display.contains("special chars"));
    }

    #[test]
    fn test_error_collector_shared_state() {
        let collector1 = ErrorCollector::new();
        let collector2 = collector1.clone();

        // Add error through first collector
        collector1.add(ExecutionError::new(
            "task_1".to_string(),
            "Error 1".to_string(),
        ));

        // Second collector should see the error
        assert_eq!(collector2.len(), 1);
        assert!(collector2.has_errors());

        // Add error through second collector
        collector2.add(ExecutionError::new(
            "task_2".to_string(),
            "Error 2".to_string(),
        ));

        // Both should see both errors
        assert_eq!(collector1.len(), 2);
        assert_eq!(collector2.len(), 2);
    }
}
