//! Configuration for the threading module.
//!
//! This module provides configuration options for the thread pool, including
//! environment variable parsing and validation.

use log::{info, warn};
use std::env;

/// Configuration for the thread pool.
///
/// This struct controls how many threads the thread pool will spawn.
/// Thread count can be configured via the `PLANTUML_GENERATOR_THREADS` environment
/// variable.
///
/// # Examples
///
/// ```ignore
/// use crate::threading::Config;
///
/// // Use default configuration (CPU core count)
/// let config = Config::default();
///
/// // Load from environment variable
/// let config = Config::from_env();
///
/// // Create with specific thread count
/// let config = Config::new(4);
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// Number of worker threads to spawn.
    thread_count: usize,
}

impl Config {
    /// Creates a new configuration with the specified thread count.
    ///
    /// # Arguments
    ///
    /// * `thread_count` - Number of worker threads (must be between 1 and 256)
    ///
    /// # Panics
    ///
    /// Panics if `thread_count` is 0 or greater than 256.
    pub fn new(thread_count: usize) -> Self {
        assert!(
            thread_count > 0 && thread_count <= 256,
            "Thread count must be between 1 and 256"
        );
        Self { thread_count }
    }

    /// Returns the configured thread count.
    pub fn thread_count(&self) -> usize {
        self.thread_count
    }

    /// Creates configuration from the environment.
    ///
    /// Reads the `PLANTUML_GENERATOR_THREADS` environment variable. If not set or
    /// invalid, falls back to the default (CPU core count).
    ///
    /// # Environment Variables
    ///
    /// * `PLANTUML_GENERATOR_THREADS` - Number of threads (1-256)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::threading::Config;
    ///
    /// // With environment variable set: PLANTUML_GENERATOR_THREADS=8
    /// let config = Config::from_env();
    /// assert_eq!(config.thread_count(), 8);
    /// ```
    pub fn from_env() -> Self {
        const ENV_VAR: &str = "PLANTUML_GENERATOR_THREADS";

        match env::var(ENV_VAR) {
            Ok(val) => match val.parse::<usize>() {
                Ok(count) if (1..=256).contains(&count) => {
                    info!(
                        "Using {} threads from environment variable {}",
                        count, ENV_VAR
                    );
                    Self::new(count)
                }
                Ok(count) => {
                    warn!(
                        "Invalid thread count {} from {}: must be 1-256. Using default.",
                        count, ENV_VAR
                    );
                    Self::default()
                }
                Err(e) => {
                    warn!(
                        "Failed to parse {} value '{}': {}. Using default.",
                        ENV_VAR, val, e
                    );
                    Self::default()
                }
            },
            Err(_) => {
                info!(
                    "Environment variable {} not set, using default thread count",
                    ENV_VAR
                );
                Self::default()
            }
        }
    }
}

impl Default for Config {
    /// Creates a default configuration using the number of logical CPU cores.
    fn default() -> Self {
        let thread_count = Self::detect_cpu_count();
        info!("Default thread count: {} (CPU cores)", thread_count);
        Self { thread_count }
    }
}

impl Config {
    /// Detects the number of logical CPU cores.
    ///
    /// In production, uses the `num_cpus` crate. In tests, returns a fixed value.
    #[cfg(not(test))]
    fn detect_cpu_count() -> usize {
        num_cpus::get()
    }

    #[cfg(test)]
    fn detect_cpu_count() -> usize {
        4 // Use fixed value in tests for deterministic behavior
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_new_valid() {
        let config = Config::new(8);
        assert_eq!(config.thread_count(), 8);
    }

    #[test]
    #[should_panic(expected = "Thread count must be between 1 and 256")]
    fn test_new_zero() {
        Config::new(0);
    }

    #[test]
    #[should_panic(expected = "Thread count must be between 1 and 256")]
    fn test_new_too_large() {
        Config::new(257);
    }

    #[test]
    fn test_default() {
        let config = Config::default();
        assert_eq!(config.thread_count(), 4); // Using test mock value
    }

    #[test]
    #[serial]
    fn test_from_env_not_set() {
        env::remove_var("PLANTUML_GENERATOR_THREADS");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 4); // Default in tests
    }

    #[test]
    #[serial]
    fn test_from_env_valid() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "16");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 16);
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    #[serial]
    fn test_from_env_invalid_falls_back() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "invalid");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 4); // Falls back to default
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    #[serial]
    fn test_from_env_out_of_range() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "300");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 4); // Falls back to default
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    fn test_new_boundary_min() {
        let config = Config::new(1);
        assert_eq!(config.thread_count(), 1);
    }

    #[test]
    fn test_new_boundary_max() {
        let config = Config::new(256);
        assert_eq!(config.thread_count(), 256);
    }

    #[test]
    #[serial]
    fn test_from_env_boundary_min() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "1");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 1);
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    #[serial]
    fn test_from_env_boundary_max() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "256");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 256);
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    #[serial]
    fn test_from_env_zero() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "0");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 4); // Falls back to default
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    #[serial]
    fn test_from_env_negative() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "-5");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 4); // Falls back to default
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    #[serial]
    fn test_from_env_empty_string() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 4); // Falls back to default
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    #[serial]
    fn test_from_env_whitespace() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "  8  ");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 4); // Falls back to default (parse fails on whitespace)
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    #[serial]
    fn test_from_env_float() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "3.14");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 4); // Falls back to default
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    #[serial]
    fn test_from_env_very_large_number() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "999999999999999999");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 4); // Falls back to default (parse fails or out of range)
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    #[serial]
    fn test_from_env_special_chars() {
        env::set_var("PLANTUML_GENERATOR_THREADS", "8!@#");
        let config = Config::from_env();
        assert_eq!(config.thread_count(), 4); // Falls back to default
        env::remove_var("PLANTUML_GENERATOR_THREADS");
    }

    #[test]
    fn test_config_clone() {
        let config1 = Config::new(10);
        let config2 = config1.clone();
        assert_eq!(config1.thread_count(), config2.thread_count());
    }

    #[test]
    fn test_detect_cpu_count_in_test() {
        let count = Config::detect_cpu_count();
        assert_eq!(count, 4); // Should be fixed value in test mode
    }
}
