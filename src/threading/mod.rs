//! Unified threading framework for parallel work execution.
//!
//! This module provides a reusable thread pool abstraction for parallelizing
//! work across CLI commands. It includes:
//!
//! - **[`WorkUnit`]** trait: Interface for parallelizable work
//! - **[`ThreadPool`]**: Manages worker threads and distributes work
//! - **[`Config`]**: Configuration with environment variable support
//! - **Error types**: [`ExecutionError`], [`AggregatedError`], and [`ErrorCollector`] for error aggregation
//!
//! # Architecture
//!
//! The threading module uses a simple but effective architecture:
//!
//! 1. **Work units**: Implement the [`WorkUnit`] trait to define parallelizable tasks
//! 2. **Thread pool**: The [`ThreadPool`] spawns worker threads based on configuration
//! 3. **Work distribution**: Uses `std::sync::mpsc` channels to distribute work
//! 4. **Error collection**: Thread-safe error aggregation using [`ErrorCollector`]
//!
//! Worker threads process work units from a shared channel until all work is complete.
//! The pool automatically joins all threads and collects any errors that occurred.
//!
//! # Basic Usage
//!
//! ## Simple Example
//!
//! ```ignore
//! use crate::threading::{Config, ThreadPool, WorkUnit};
//!
//! // Define a work unit
//! struct FileProcessor {
//!     file_path: String,
//! }
//!
//! impl WorkUnit for FileProcessor {
//!     fn identifier(&self) -> String {
//!         self.file_path.clone()
//!     }
//!
//!     fn execute(&self) -> Result<(), String> {
//!         // Process the file
//!         println!("Processing: {}", self.file_path);
//!         // Your processing logic here
//!         Ok(())
//!     }
//! }
//!
//! // Use the thread pool
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create configuration from environment (PLANTUML_GENERATOR_THREADS)
//!     let config = Config::from_env();
//!     let pool = ThreadPool::new(config);
//!     
//!     // Create work units
//!     let files = vec!["file1.txt", "file2.txt", "file3.txt"];
//!     let tasks: Vec<Box<dyn WorkUnit>> = files
//!         .into_iter()
//!         .map(|file| Box::new(FileProcessor { 
//!             file_path: file.to_string() 
//!         }) as Box<dyn WorkUnit>)
//!         .collect();
//!     
//!     // Execute all tasks in parallel
//!     pool.execute(tasks)?;
//!     println!("All files processed successfully!");
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling Example
//!
//! ```ignore
//! use crate::threading::{Config, ThreadPool, WorkUnit, AggregatedError};
//!
//! struct DataValidator {
//!     data_id: usize,
//!     data: String,
//! }
//!
//! impl WorkUnit for DataValidator {
//!     fn identifier(&self) -> String {
//!         format!("validator_{}", self.data_id)
//!     }
//!
//!     fn execute(&self) -> Result<(), String> {
//!         if self.data.is_empty() {
//!             return Err(format!("Data {} is empty", self.data_id));
//!         }
//!         // Validation logic
//!         Ok(())
//!     }
//! }
//!
//! fn validate_data() {
//!     let pool = ThreadPool::new(Config::new(4));
//!     let tasks: Vec<Box<dyn WorkUnit>> = vec![
//!         Box::new(DataValidator { data_id: 1, data: "valid".to_string() }),
//!         Box::new(DataValidator { data_id: 2, data: "".to_string() }), // Will fail
//!         Box::new(DataValidator { data_id: 3, data: "valid".to_string() }),
//!     ];
//!     
//!     match pool.execute(tasks) {
//!         Ok(()) => println!("All validations passed!"),
//!         Err(agg) => {
//!             eprintln!("Validation failed with {} errors:", agg.len());
//!             for error in agg.errors() {
//!                 eprintln!("  - {}", error);
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Configuration Examples
//!
//! ```ignore
//! use crate::threading::Config;
//!
//! // Default configuration (CPU core count)
//! let config_default = Config::default();
//!
//! // Explicit thread count
//! let config_explicit = Config::new(8);
//!
//! // From environment variable PLANTUML_GENERATOR_THREADS
//! let config_env = Config::from_env();
//!
//! println!("Thread count: {}", config_env.thread_count());
//! ```
//!
//! # Configuration
//!
//! The thread pool can be configured in three ways:
//!
//! 1. **Environment variable**: Set `PLANTUML_GENERATOR_THREADS` (1-256)
//! 2. **Explicit count**: Use [`Config::new(count)`](Config::new)
//! 3. **Default**: Use [`Config::default()`](Config::default) for CPU core count
//!
//! Environment variable example:
//! ```bash
//! export PLANTUML_GENERATOR_THREADS=8
//! plantuml-generator library generate
//! ```
//!
//! Invalid values fall back to the default (CPU core count). The thread pool
//! automatically limits threads to the number of work units when there are
//! fewer work units than configured threads.
//!
//! # Error Handling
//!
//! The thread pool aggregates errors from work unit execution into an
//! [`AggregatedError`]. This error type:
//!
//! - Collects all errors from failed work units
//! - Provides access to individual errors via [`errors()`](AggregatedError::errors)
//! - Displays a formatted summary of all failures
//! - Continues executing remaining work units after failures
//!
//! Worker thread panics are caught and converted to errors with the worker ID.
//!
//! # Performance Characteristics
//!
//! - **Thread overhead**: Worker threads are spawned once and reused for all work
//! - **Work distribution**: Lock-free channel with minimal contention
//! - **Scalability**: Linear speedup up to CPU core count for CPU-bound work
//! - **Thread safety**: All types are `Send` and use appropriate synchronization
//!
//! For I/O-bound work, you may benefit from more threads than CPU cores.
//! For CPU-bound work, the default (CPU core count) is usually optimal.
//!
//! # Thread Safety
//!
//! - [`WorkUnit`] must implement `Send + 'static` for safe transfer between threads
//! - [`ErrorCollector`] uses `Arc<Mutex<>>` for thread-safe error aggregation
//! - All public APIs are thread-safe and can be used from multiple threads
//!
//! # See Also
//!
//! - [`WorkUnit`]: Trait for defining parallelizable work
//! - [`ThreadPool`]: The main thread pool implementation
//! - [`Config`]: Thread pool configuration options
//! - [`AggregatedError`]: Error aggregation for multiple failures

// Allow dead code since this is a foundational module being built incrementally
#![allow(dead_code)]
#![allow(unused_imports)]

mod config;
mod errors;
mod pool;
mod traits;

// Re-export public API
pub use config::Config;
pub use errors::{AggregatedError, ErrorCollector, ExecutionError};
pub use pool::ThreadPool;
pub use traits::WorkUnit;
