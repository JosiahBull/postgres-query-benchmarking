mod any_array;
mod chunked_prepared;
mod raw_sql_large_in;
mod temp_table_any;
mod temp_table_binary_copy;
mod temp_table_binary_no_index;
mod temp_table_join;
mod temp_table_optimized_binary;
mod temp_table_text_copy;
mod unnest_array;

// Re-export all benchmark implementations
pub use any_array::AnyArrayBenchmark;
pub use chunked_prepared::ChunkedPreparedBenchmark;
pub use raw_sql_large_in::RawSqlLargeInBenchmark;
pub use temp_table_any::TempTableAnyBenchmark;
pub use temp_table_binary_copy::TempTableBinaryCopyBenchmark;
pub use temp_table_binary_no_index::TempTableBinaryNoIndexBenchmark;
pub use temp_table_join::TempTableJoinBenchmark;
pub use temp_table_optimized_binary::TempTableOptimizedBinaryBenchmark;
pub use temp_table_text_copy::TempTableTextCopyBenchmark;
pub use unnest_array::UnnestArrayBenchmark;

use crate::BenchmarkTest;
use std::sync::Arc;

/// Get all available benchmarks
pub fn get_all_benchmarks() -> Vec<Arc<dyn BenchmarkTest>> {
    vec![
        Arc::new(ChunkedPreparedBenchmark),
        Arc::new(AnyArrayBenchmark),
        Arc::new(UnnestArrayBenchmark),
        Arc::new(TempTableTextCopyBenchmark),
        Arc::new(TempTableBinaryCopyBenchmark),
        Arc::new(TempTableOptimizedBinaryBenchmark),
        Arc::new(TempTableJoinBenchmark),
        Arc::new(TempTableAnyBenchmark),
        Arc::new(RawSqlLargeInBenchmark),
        Arc::new(TempTableBinaryNoIndexBenchmark),
    ]
}

/// Get benchmark by name
pub fn get_benchmark_by_name(name: &str) -> Option<Arc<dyn BenchmarkTest>> {
    get_all_benchmarks()
        .into_iter()
        .find(|benchmark| benchmark.name() == name)
}
