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
