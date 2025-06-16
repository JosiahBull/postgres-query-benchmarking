use crate::{BenchmarkContext, BenchmarkError, BenchmarkResult, BenchmarkTest, ExampleData};
use async_trait::async_trait;
use tracing::instrument;

/// Benchmark that uses raw SQL with large IN clause
pub struct RawSqlLargeInBenchmark;

#[async_trait]
impl BenchmarkTest for RawSqlLargeInBenchmark {
    #[instrument(skip(self, context, ids), fields(ids_count = ids.len()))]
    async fn run(
        &self,
        context: &BenchmarkContext,
        ids: &[i64],
    ) -> BenchmarkResult<Vec<ExampleData>> {
        // Build the IN clause string directly to eliminate parameter binding overhead
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Construct the complete SQL query
        let query = format!(
            "SELECT RESPONSE as response FROM OVERRIDES WHERE HASH IN ({});",
            ids_str
        );

        // Execute the raw SQL query
        let result: Vec<ExampleData> = sqlx::query_as(&query)
            .fetch_all(&context.pool)
            .await
            .map_err(BenchmarkError::Database)?;

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "raw_sql_large_in"
    }

    fn description(&self) -> &'static str {
        "Builds large IN clause as raw SQL string to eliminate network/parameter binding overhead"
    }

    async fn cleanup(&self, context: &BenchmarkContext) -> BenchmarkResult<()> {
        // Clear caches after the benchmark
        context.clear_caches().await?;
        Ok(())
    }
}
