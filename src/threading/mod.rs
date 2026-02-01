//! Unified threading framework for parallel work execution.
//!
//! This module provides a reusable thread pool abstraction for parallelizing
//! work across CLI commands. It includes:
//!
//! - **`WorkUnit`** trait: Interface for parallelizable work
//! - **`ThreadPool`**: Manages worker threads and distributes work
//! - **`Config`**: Configuration with environment variable support
//! - **Error types**: For error aggregation and fail-fast handling
//!
//! # Examples
//!
//! ```ignore
//! use crate::threading::{Config, ThreadPool, WorkUnit};
//!
//! // Define a work unit
//! struct MyTask {
//!     id: usize,
//!     data: String,
//! }
//!
//! impl WorkUnit for MyTask {
//!     fn identifier(&self) -> String {
//!         format!("task_{}", self.id)
//!     }
//!
//!     fn execute(&self) -> Result<(), String> {
//!         // Perform work
//!         println!("Processing: {}", self.data);
//!         Ok(())
//!     }
//! }
//!
//! // Use the thread pool
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::from_env();
//!     let pool = ThreadPool::new(config);
//!     
//!     let tasks: Vec<Box<dyn WorkUnit>> = vec![
//!         Box::new(MyTask { id: 1, data: "data1".to_string() }),
//!         Box::new(MyTask { id: 2, data: "data2".to_string() }),
//!     ];
//!     
//!     pool.execute(tasks)?;
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! The thread pool can be configured via the `PLANTUML_GENERATOR_THREADS` environment
//! variable. Valid values are 1-256. If not set or invalid, defaults to the
//! number of CPU cores.
//!
//! # Error Handling
//!
//! The thread pool aggregates errors from work unit execution into an
//! `AggregatedError`. A future implementation may add optional fail-fast
//! behavior (stopping remaining work when the first failure occurs).

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
