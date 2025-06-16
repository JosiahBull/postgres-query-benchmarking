use crate::{BenchmarkContext, BenchmarkError, BenchmarkResult, BenchmarkTest, ExampleData};
use async_trait::async_trait;
use tracing::instrument;

/// Benchmark that uses UNNEST function with array parameter
pub struct UnnestArrayBenchmark;

#[async_trait]
impl BenchmarkTest for UnnestArrayBenchmark {
    #[instrument(skip(self, context, ids), fields(ids_count = ids.len()))]
    async fn run(
        &self,
        context: &BenchmarkContext,
        ids: &[i64],
    ) -> BenchmarkResult<Vec<ExampleData>> {
        let result: Vec<ExampleData> = sqlx::query_as(
            "SELECT RESPONSE as response FROM OVERRIDES WHERE HASH IN (SELECT UNNEST($1::bigint[]));",
        )
        .bind(ids)
        .fetch_all(&context.pool)
        .await
        .map_err(BenchmarkError::Database)?;

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "unnest_array"
    }

    fn description(&self) -> &'static str {
        "Uses PostgreSQL's UNNEST function to convert array to table"
    }
}
