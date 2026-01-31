//! Error types for the threading module.
//!
//! This module defines error types for thread pool execution, including
//! error aggregation for collecting failures from multiple work units.

use std::fmt;

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
}
