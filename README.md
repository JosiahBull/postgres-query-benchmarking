# PostgreSQL Query Benchmarking Suite

This project benchmarks different approaches for querying PostgreSQL with large sets of IDs, comparing performance characteristics of various strategies.

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

## Prerequisites

- Rust (latest stable)
- Docker and Docker Compose
- PostgreSQL client tools (`psql`)

## Quick Start

The easiest way to get started is using the provided Docker-based setup:

```bash
# Initialize directories and start PostgreSQL container
./scripts/setup.sh

# Run benchmarks
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

The suite generates detailed results in `benchmark_results.log` containing:

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

### Performance Notes

- Run benchmarks with `--release` flag for accurate timing
- The Docker container is configured with cold query settings for consistent benchmarking
- JIT compilation and plan caching are disabled to ensure true cold query performance

## License

All code is licensed under the MIT License. See the [LICENSE](LICENSE) file for
details.

## Contribution

Contributions are welcome! Please open issues or pull requests for any
improvements, bug fixes, or new benchmarks. Unless stated otherwise, all
contributions will be considered under the MIT License.
