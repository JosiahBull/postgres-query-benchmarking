# PostgreSQL Query Benchmarking Suite

This project benchmarks different approaches for querying PostgreSQL with large sets of IDs, comparing performance characteristics of various strategies. Results are automatically exported to CSV format for easy analysis in Excel, Google Sheets, or other data analysis tools.

## Overview

The benchmarking suite tests different methods for executing queries with large ID sets (60,000 IDs) against a PostgreSQL database:

1. **Chunked Prepared Statements** - Splits IDs into chunks and uses prepared statements with placeholders
2. **ANY Array** - Uses PostgreSQL's ANY operator with array parameters
3. **UNNEST Array** - Uses UNNEST function with array parameters
4. **Temporary Table (Text COPY)** - Creates temp table and uses COPY with text format
5. **Temporary Table (Binary COPY)** - Creates temp table and uses COPY with binary format
6. **Temporary Table (Optimized Binary)** - Optimized binary COPY with storage settings
7. **Temporary Table with JOIN** - Uses JOIN instead of IN clause with temp table
8. **Temporary Table with ANY** - Uses ANY operator with temp table subquery
9. **Raw SQL Large IN** - Builds large IN clause string to eliminate network overhead

## Results

```
PostgreSQL Query Benchmark Results
==================================
Timestamp: 2025-06-18 22:43:10 UTC

Benchmark                               Runs       Median         Mean          Min          Max       StdDev     Rows InputSize
--------------------------------------------------------------------------------------------------------------------------------------------
temp_table_any                           100     174.96ms     174.89ms     164.15ms     196.25ms       6.16ms    60000    60000
temp_table_join                          100     176.61ms     179.28ms     166.54ms     224.13ms      10.91ms    60000    60000
temp_table_optimized_binary              100     177.46ms     180.10ms     165.79ms     225.36ms      10.02ms    60000    60000
temp_table_binary_copy                   100     180.77ms     182.97ms     167.46ms     216.38ms      11.00ms    60000    60000
any_array                                100     224.14ms     225.49ms     215.03ms     257.27ms       7.67ms    60000    60000
unnest_array                             100     272.67ms     276.71ms     259.85ms     326.24ms      13.19ms    60000    60000
raw_sql_large_in                         100     275.19ms     278.87ms     262.49ms     317.07ms      11.10ms    60000    60000
temp_table_binary_no_index               100     280.47ms     288.43ms     269.69ms     388.50ms      20.48ms    60000    60000
chunked_prepared                         100     298.36ms     301.71ms     279.73ms     336.27ms      12.59ms    60000    60000
```

## Quick Start

The easiest way to get started is using the provided Docker-based setup:

```bash
# Initialize directories and start PostgreSQL container
./scripts/setup.sh

# Or with optimizations enabled
./scripts/setup.sh --mode optimized

# Run benchmarks with CSV output (default)
cargo run --release
```

## Running the Benchmarks

```bash
# Run all benchmarks with default settings (100 iterations per benchmark)
cargo run --release

# List available benchmarks
cargo run --release -- --list

# Run a specific benchmark
cargo run --release -- chunked_prepared

# Check compilation
cargo check
```

## Output

The suite generates multiple output formats:

### CSV Files (Default)
- **`logs/raw_results.csv`** - Individual timing data for each benchmark run
  - Columns: benchmark_name, description, input_size, rows_returned, run_number, duration_ms, duration_ns
  - Perfect for time-series analysis and detailed statistical work
- **`logs/summary.csv`** - Aggregated statistics for each benchmark
  - Columns: benchmark_name, description, input_size, rows_returned, total_runs, mean_ms, median_ms, std_dev_ms, min_ms, max_ms, p50_ms, p95_ms, p99_ms

### Log File
The suite also generates detailed results in `logs/benchmark_results.log` containing:

- **Summary Table** - Sorted by median performance
- **Detailed Statistics** - Mean, median, min, max, standard deviation
- **Percentiles** - 95th and 99th percentile timings
- **Metadata** - Number of rows returned, input size, run count

### Adding New Benchmarks

1. Create a new benchmark file in `src/benchmarks/`:

```rust
// src/benchmarks/my_approach.rs
use crate::{BenchmarkContext, BenchmarkError, BenchmarkResult, BenchmarkTest, ExampleData};
use async_trait::async_trait;

pub struct MyApproachBenchmark;

#[async_trait]
impl BenchmarkTest for MyApproachBenchmark {
    async fn run(
        &self,
        context: &BenchmarkContext,
        ids: &[i64],
    ) -> BenchmarkResult<Vec<ExampleData>> {
        // Your implementation here
        todo!()
    }

    fn name(&self) -> &'static str {
        "my_approach"
    }

    fn description(&self) -> &'static str {
        "Description of my approach"
    }
}
```

2. Add it to `src/benchmarks/mod.rs`:

```rust
pub mod my_approach;
pub use my_approach::MyApproachBenchmark;

// Add to get_all_benchmarks() function:
Arc::new(MyApproachBenchmark),
```

## License

All code is licensed under the MIT License. See the [LICENSE](LICENSE) file for
details.

## Contribution

Contributions are welcome! Please open issues or pull requests for any
improvements, bug fixes, or new benchmarks. Unless stated otherwise, all
contributions will be considered under the MIT License.
