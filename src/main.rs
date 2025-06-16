//! PostgreSQL Query Benchmarking Suite
//!
//! This is the main entry point for the PostgreSQL query benchmarking suite.
//! It runs various benchmarks against different query strategies and generates
//! detailed performance statistics.

use pg_hacking::{
    BenchmarkContext, BenchmarkStats, BenchmarkTest, ID_RANGE, ITERATIONS, LOG_FILE_NAME,
    MAX_CONNECTIONS, TEST_IDS,
    benchmarks::{get_all_benchmarks, get_benchmark_by_name},
    utils::generate_test_ids,
};

use clap::{Parser, Subcommand};
use sqlx::postgres::PgPoolOptions;
use std::{fs::File, io::Write, sync::Arc, time::Instant};
use tracing::{error, info, instrument, warn};

/// Command line arguments for the benchmark suite
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Database URL (can also be set via DATABASE_URL environment variable)
    #[arg(short, long)]
    database_url: Option<String>,

    /// Number of iterations to run each benchmark
    #[arg(short, long, default_value_t = ITERATIONS)]
    iterations: usize,

    /// Number of test IDs to generate
    #[arg(short, long, default_value_t = TEST_IDS)]
    test_ids: usize,

    /// Command to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available benchmarks
    List,
    /// Run a specific benchmark by name
    Run {
        /// Benchmark name to run
        name: String,
    },
}

/// Benchmark suite for running and collecting results
struct BenchmarkSuite {
    context: BenchmarkContext,
    results: Vec<BenchmarkStats>,
    log_file: File,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Connecting to database: {}", database_url);
        let pool = PgPoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect(database_url)
            .await
            .map_err(|e| {
                error!("Failed to connect to database: {}", e);
                e
            })?;

        let context = BenchmarkContext::new(pool);

        info!("Creating log file: {}", LOG_FILE_NAME);
        let log_file = File::create(LOG_FILE_NAME).map_err(|e| {
            error!("Failed to create log file: {}", e);
            e
        })?;

        Ok(Self {
            context,
            results: Vec::new(),
            log_file,
        })
    }

    /// Run a single benchmark with multiple iterations
    #[instrument(skip(self, benchmark, ids))]
    async fn run_benchmark(
        &mut self,
        benchmark: Arc<dyn BenchmarkTest>,
        ids: &[i64],
        iterations: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = benchmark.name();
        let description = benchmark.description();

        info!(
            "Starting benchmark: {} with {} iterations",
            name, iterations
        );

        let mut stats = BenchmarkStats::new(name.to_string(), description.to_string(), ids.len());

        // Warmup run (only if benchmark needs it)
        if benchmark.needs_warmup() {
            info!("Warming up benchmark: {}", name);
            match benchmark.run(&self.context, ids).await {
                Ok(_) => info!("Warmup completed for: {}", name),
                Err(e) => {
                    error!("Warmup failed for {}: {}", name, e);
                    return Ok(());
                }
            }
        }

        // Run benchmark iterations
        for i in 0..iterations {
            // Clear caches before each run for cold query performance
            if let Err(e) = self.context.clear_caches().await {
                warn!(
                    "Failed to clear caches for {} iteration {}: {}",
                    name,
                    i + 1,
                    e
                );
            }

            let start = Instant::now();
            match benchmark.run(&self.context, ids).await {
                Ok(results) => {
                    let duration = start.elapsed();
                    stats.add_result(duration, results.len());
                    info!(
                        "Benchmark {} iteration {}/{} completed in {:?} ({} rows)",
                        name,
                        i + 1,
                        iterations,
                        duration,
                        results.len()
                    );
                }
                Err(e) => {
                    warn!(
                        "Benchmark {} iteration {}/{} failed: {}",
                        name,
                        i + 1,
                        iterations,
                        e
                    );
                    continue;
                }
            }
        }

        // Cleanup benchmark
        if let Err(e) = benchmark.cleanup(&self.context).await {
            warn!("Cleanup failed for {}: {}", name, e);
        }

        if !stats.runs.is_empty() {
            let runs_count = stats.runs.len();
            self.results.push(stats);
            info!(
                "Benchmark {} completed: {} successful runs out of {} attempts",
                name, runs_count, iterations
            );
        } else {
            warn!("Benchmark {} had no successful runs", name);
        }

        Ok(())
    }

    /// Write benchmark results to log file
    fn write_results(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(self.log_file, "PostgreSQL Query Benchmark Results")?;
        writeln!(self.log_file, "==================================")?;
        writeln!(
            self.log_file,
            "Timestamp: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;
        writeln!(self.log_file)?;

        // Sort by median time for easy comparison
        self.results.sort_by_key(|r| r.median());

        // Summary table
        writeln!(
            self.log_file,
            "{:<35} {:>8} {:>12} {:>12} {:>12} {:>12} {:>12} {:>8} {:>8}",
            "Benchmark", "Runs", "Median", "Mean", "Min", "Max", "StdDev", "Rows", "InputSize"
        )?;
        writeln!(self.log_file, "{}", "-".repeat(140))?;

        for result in &self.results {
            writeln!(
                self.log_file,
                "{:<35} {:>8} {:>12.2?} {:>12.2?} {:>12.2?} {:>12.2?} {:>12.2?} {:>8} {:>8}",
                result.name,
                result.runs.len(),
                result.median(),
                result.mean(),
                result.min(),
                result.max(),
                result.std_deviation(),
                result.rows_returned,
                result.input_size
            )?;
        }

        writeln!(self.log_file)?;
        writeln!(self.log_file, "Detailed Statistics:")?;
        writeln!(self.log_file, "===================")?;

        for result in &self.results {
            writeln!(self.log_file)?;
            writeln!(
                self.log_file,
                "Benchmark: {} ({})",
                result.name, result.description
            )?;
            writeln!(self.log_file, "  Runs: {}", result.runs.len())?;
            writeln!(self.log_file, "  Input Size: {} IDs", result.input_size)?;
            writeln!(self.log_file, "  Rows Returned: {}", result.rows_returned)?;
            writeln!(self.log_file, "  Median: {:?}", result.median())?;
            writeln!(self.log_file, "  Mean: {:?}", result.mean())?;
            writeln!(self.log_file, "  Min: {:?}", result.min())?;
            writeln!(self.log_file, "  Max: {:?}", result.max())?;
            writeln!(
                self.log_file,
                "  Standard Deviation: {:?}",
                result.std_deviation()
            )?;

            // Percentiles
            writeln!(
                self.log_file,
                "  95th Percentile: {:?}",
                result.percentile(95.0)
            )?;
            writeln!(
                self.log_file,
                "  99th Percentile: {:?}",
                result.percentile(99.0)
            )?;
        }

        writeln!(self.log_file)?;
        writeln!(self.log_file, "Performance Ranking (by median time):")?;
        writeln!(self.log_file, "=====================================")?;
        for (i, result) in self.results.iter().enumerate() {
            writeln!(
                self.log_file,
                "{}. {} - {:?}",
                i + 1,
                result.name,
                result.median()
            )?;
        }

        self.log_file.flush()?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if it exists
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Get database URL from CLI argument, environment variable, or use default
    let database_url = cli
        .database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| "postgres://postgres:postgres@localhost".to_string());

    // Handle list command early
    if let Some(Commands::List) = cli.command {
        println!("Available benchmarks:");
        for benchmark in get_all_benchmarks() {
            println!("  {}: {}", benchmark.name(), benchmark.description());
        }
        return Ok(());
    }

    // Initialize benchmark suite
    let mut suite = BenchmarkSuite::new(&database_url).await?;

    // Generate test data
    info!(
        "Generating {} unique random IDs between 1 and {}",
        cli.test_ids, ID_RANGE
    );
    let ids = generate_test_ids(cli.test_ids, ID_RANGE);
    info!("Generated {} unique IDs for testing", ids.len());

    // Select benchmarks based on command
    let benchmarks = match cli.command {
        None => {
            info!("Running all benchmarks");
            get_all_benchmarks()
        }
        Some(Commands::List) => {
            // Already handled above
            unreachable!()
        }
        Some(Commands::Run { name }) => {
            if let Some(benchmark) = get_benchmark_by_name(&name) {
                info!("Running single benchmark: {}", name);
                vec![benchmark]
            } else {
                error!("Benchmark not found: {}", name);
                return Ok(());
            }
        }
    };

    if benchmarks.is_empty() {
        warn!("No benchmarks selected to run");
        return Ok(());
    }

    info!("Selected {} benchmarks to run", benchmarks.len());

    // Run all selected benchmarks
    for benchmark in benchmarks {
        if let Err(e) = suite.run_benchmark(benchmark, &ids, cli.iterations).await {
            error!("Failed to run benchmark: {}", e);
        }
    }

    // Write results
    info!("Writing benchmark results...");
    suite.write_results()?;

    info!("Benchmark completed! Results written to {}", LOG_FILE_NAME);
    info!("Total benchmarks completed: {}", suite.results.len());

    // Print summary to console
    println!("\nBenchmark Summary:");
    println!("==================");
    for (i, result) in suite.results.iter().enumerate() {
        println!(
            "{}. {} - {:?} median ({} runs)",
            i + 1,
            result.name,
            result.median(),
            result.runs.len()
        );
    }

    Ok(())
}
