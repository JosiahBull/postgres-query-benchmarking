//! PostgreSQL Query Benchmarking Library
//!
//! This library provides a common interface for benchmarking different PostgreSQL query strategies
//! with large sets of IDs. Each benchmark implements the `BenchmarkTest` trait to ensure
//! consistent testing and measurement.

use async_trait::async_trait;
use sqlx::postgres::PgPool;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use thiserror::Error;

/// Configuration constants for benchmarking
pub const ITERATIONS: usize = 1000;
pub const TEST_IDS: usize = 60_000;
pub const ID_RANGE: u64 = 20_000_000;
pub const MAX_CONNECTIONS: u32 = 10;
pub const LOG_FILE_NAME: &str = "logs/benchmark_results.log";
pub const CSV_FILE_NAME: &str = "logs/benchmark_results.csv";

/// Data structure returned by benchmark queries
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ExampleData {
    pub response: String,
}

/// Custom error types for benchmarking
#[derive(Error, Debug)]
pub enum BenchmarkError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Benchmark failed: {message}")]
    BenchmarkFailed { message: String },
    #[error("Setup error: {message}")]
    Setup { message: String },
}

/// Result type for benchmark operations
pub type BenchmarkResult<T> = Result<T, BenchmarkError>;

/// Statistics collected for each benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub name: String,
    pub description: String,
    pub runs: Vec<Duration>,
    pub rows_returned: usize,
    pub input_size: usize,
}

impl BenchmarkStats {
    /// Create new benchmark statistics
    pub fn new(name: String, description: String, input_size: usize) -> Self {
        Self {
            name,
            description,
            runs: Vec::new(),
            rows_returned: 0,
            input_size,
        }
    }

    /// Export raw timing data to CSV format
    ///
    /// # Arguments
    /// * `csv_path` - Path to the CSV file to write to
    ///
    /// # Returns
    /// * `BenchmarkResult<()>` - Success or IO error
    pub fn export_to_csv(&self, csv_path: &Path) -> BenchmarkResult<()> {
        let file_exists = csv_path.exists();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(csv_path)?;

        // Write header if file is new
        if !file_exists {
            writeln!(
                file,
                "benchmark_name,description,input_size,rows_returned,run_number,duration_ms,duration_ns"
            )?;
        }

        // Write each run as a separate row
        for (run_number, duration) in self.runs.iter().enumerate() {
            writeln!(
                file,
                "{},{},{},{},{},{},{}",
                self.name,
                self.description.replace(",", ";"), // Replace commas to avoid CSV issues
                self.input_size,
                self.rows_returned,
                run_number + 1,
                duration.as_millis(),
                duration.as_nanos()
            )?;
        }

        Ok(())
    }

    /// Export summary statistics to CSV format
    ///
    /// # Arguments
    /// * `csv_path` - Path to the CSV file to write to
    ///
    /// # Returns
    /// * `BenchmarkResult<()>` - Success or IO error
    pub fn export_summary_to_csv(&self, csv_path: &Path) -> BenchmarkResult<()> {
        let file_exists = csv_path.exists();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(csv_path)?;

        // Write header if file is new
        if !file_exists {
            writeln!(
                file,
                "benchmark_name,description,input_size,rows_returned,total_runs,mean_ms,median_ms,std_dev_ms,min_ms,max_ms,p50_ms,p95_ms,p99_ms"
            )?;
        }

        // Write summary statistics
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.name,
            self.description.replace(",", ";"), // Replace commas to avoid CSV issues
            self.input_size,
            self.rows_returned,
            self.runs.len(),
            self.mean().as_millis(),
            self.median().as_millis(),
            self.std_deviation().as_millis(),
            self.min().as_millis(),
            self.max().as_millis(),
            self.percentile(50.0).as_millis(),
            self.percentile(95.0).as_millis(),
            self.percentile(99.0).as_millis()
        )?;

        Ok(())
    }

    /// Add a benchmark result
    pub fn add_result(&mut self, duration: Duration, rows_returned: usize) {
        self.runs.push(duration);
        self.rows_returned = rows_returned; // Assume consistent across runs
    }

    /// Calculate mean duration
    pub fn mean(&self) -> Duration {
        if self.runs.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.runs.iter().sum();
        total / self.runs.len() as u32
    }

    /// Calculate median duration
    pub fn median(&self) -> Duration {
        if self.runs.is_empty() {
            return Duration::ZERO;
        }
        let mut sorted = self.runs.clone();
        sorted.sort();
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2
        } else {
            sorted[mid]
        }
    }

    /// Calculate standard deviation
    pub fn std_deviation(&self) -> Duration {
        if self.runs.len() < 2 {
            return Duration::ZERO;
        }
        let mean = self.mean();
        let variance: f64 = self
            .runs
            .iter()
            .map(|&d| {
                let diff = d.as_nanos() as f64 - mean.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>()
            / self.runs.len() as f64;
        Duration::from_nanos(variance.sqrt() as u64)
    }

    /// Get minimum duration
    pub fn min(&self) -> Duration {
        *self.runs.iter().min().unwrap_or(&Duration::ZERO)
    }

    /// Get maximum duration
    pub fn max(&self) -> Duration {
        *self.runs.iter().max().unwrap_or(&Duration::ZERO)
    }

    /// Get nth percentile
    pub fn percentile(&self, p: f64) -> Duration {
        if self.runs.is_empty() || !(0.0..=100.0).contains(&p) {
            return Duration::ZERO;
        }
        let mut sorted = self.runs.clone();
        sorted.sort();
        let index = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

/// Benchmark execution context containing setup and teardown information
#[derive(Debug)]
pub struct BenchmarkContext {
    pub pool: PgPool,
    pub cold_query_mode: bool,
    pub disable_cache: bool,
}

impl BenchmarkContext {
    /// Create new benchmark context
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cold_query_mode: true,
            disable_cache: true,
        }
    }

    /// Clear query plan cache and statistics
    pub async fn clear_caches(&self) -> BenchmarkResult<()> {
        if !self.disable_cache {
            return Ok(());
        }

        // Clear plan cache
        sqlx::query("DISCARD PLANS;").execute(&self.pool).await?;

        // Reset statistics
        sqlx::query("SELECT pg_stat_reset();")
            .execute(&self.pool)
            .await?;

        // Clear shared buffers (if we have permissions)
        let _ = sqlx::query("SELECT pg_reload_conf();")
            .execute(&self.pool)
            .await;

        Ok(())
    }
}

/// Main trait that all benchmark implementations must implement
#[async_trait]
pub trait BenchmarkTest: Send + Sync {
    /// Run the benchmark with the given IDs
    ///
    /// # Arguments
    /// * `context` - Benchmark execution context
    /// * `ids` - Array of IDs to query with
    ///
    /// # Returns
    /// * `Result<Vec<ExampleData>, BenchmarkError>` - Query results or error
    async fn run(
        &self,
        context: &BenchmarkContext,
        ids: &[[u8; 32]],
    ) -> BenchmarkResult<Vec<ExampleData>>;

    /// Get the name of this benchmark (used for identification)
    fn name(&self) -> &'static str;

    /// Get a description of what this benchmark tests
    fn description(&self) -> &'static str;

    /// Perform any cleanup required after running the benchmark
    ///
    /// # Arguments
    /// * `context` - Benchmark execution context
    ///
    /// # Returns
    /// * `Result<(), BenchmarkError>` - Success or cleanup error
    async fn cleanup(&self, context: &BenchmarkContext) -> BenchmarkResult<()> {
        // Default implementation: clear caches
        context.clear_caches().await
    }

    /// Whether this benchmark requires a warm-up run
    fn needs_warmup(&self) -> bool {
        false // Default: no warmup for cold query testing
    }
}

/// Benchmark implementations module
pub mod benchmarks;

/// Utility functions for benchmarking
pub mod utils {
    use super::*;
    use sha2::Digest;
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;
    use tracing::info;

    /// Initialize CSV output directory and files
    ///
    /// # Arguments
    /// * `csv_dir` - Directory to create for CSV files
    ///
    /// # Returns
    /// * `BenchmarkResult<()>` - Success or IO error
    pub fn init_csv_output(csv_dir: &Path) -> BenchmarkResult<()> {
        // Create the directory if it doesn't exist
        fs::create_dir_all(csv_dir)?;

        let raw_results_path = csv_dir.join("raw_results.csv");
        let summary_path = csv_dir.join("summary.csv");

        // Clear existing files by truncating them
        if raw_results_path.exists() {
            fs::remove_file(&raw_results_path)?;
        }
        if summary_path.exists() {
            fs::remove_file(&summary_path)?;
        }

        info!("Initialized CSV output directory: {}", csv_dir.display());
        Ok(())
    }

    /// Get the path for raw results CSV file
    ///
    /// # Arguments
    /// * `csv_dir` - Directory containing CSV files
    ///
    /// # Returns
    /// * `PathBuf` - Path to raw results CSV file
    pub fn get_raw_results_csv_path(csv_dir: &Path) -> std::path::PathBuf {
        csv_dir.join("raw_results.csv")
    }

    /// Get the path for summary CSV file
    ///
    /// # Arguments
    /// * `csv_dir` - Directory containing CSV files
    ///
    /// # Returns
    /// * `PathBuf` - Path to summary CSV file
    pub fn get_summary_csv_path(csv_dir: &Path) -> std::path::PathBuf {
        csv_dir.join("summary.csv")
    }

    /// Generate unique random IDs for testing
    ///
    /// # Arguments
    /// * `count` - Number of IDs to generate
    /// * `range` - Maximum value for ID generation
    ///
    /// # Returns
    /// * `Vec<[u8; 32]>` - Vector of unique random IDs hashed with SHA-256
    pub fn generate_test_ids(count: usize, range: u64) -> Vec<[u8; 32]> {
        info!(
            "Generating {} unique random IDs between 1 and {}",
            count, range
        );

        let mut ids = HashSet::new();
        while ids.len() < count {
            let id = rand::random::<u64>() % range;
            if id == 0 {
                continue; // Skip zero to avoid database issues
            }
            ids.insert(id as i64);
        }

        // Sha256 hash the IDs
        ids.into_iter()
            .map(|id| {
                let mut hasher = sha2::Sha256::new();
                hasher.update(id.to_string()); // Not how we would do this in production, but for parity with PG implementation
                let hash = hasher.finalize();
                let mut id_bytes = [0u8; 32];
                id_bytes.copy_from_slice(&hash);
                id_bytes
            })
            .collect::<Vec<[u8; 32]>>()
    }

    /// Validate that a benchmark result is reasonable
    ///
    /// # Arguments
    /// * `results` - Query results to validate
    /// * `max_expected` - Maximum expected number of results
    ///
    /// # Returns
    /// * `BenchmarkResult<()>` - Success or validation error
    pub fn validate_results(results: &[ExampleData], max_expected: usize) -> BenchmarkResult<()> {
        if results.len() > max_expected {
            return Err(BenchmarkError::BenchmarkFailed {
                message: format!(
                    "Too many results returned: {} > {}",
                    results.len(),
                    max_expected
                ),
            });
        }

        // Validate that all results have non-empty responses
        for (i, result) in results.iter().enumerate() {
            if result.response.is_empty() {
                return Err(BenchmarkError::BenchmarkFailed {
                    message: format!("Empty response at index {}", i),
                });
            }
        }

        Ok(())
    }
}
