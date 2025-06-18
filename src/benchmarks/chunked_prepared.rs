use crate::{BenchmarkContext, BenchmarkError, BenchmarkResult, BenchmarkTest, ExampleData};
use async_trait::async_trait;
use tracing::instrument;

/// Maximum number of values per prepared statement
const MAX_VALUES: usize = 2000;

/// Benchmark that uses chunked prepared statements
pub struct ChunkedPreparedBenchmark;

#[async_trait]
impl BenchmarkTest for ChunkedPreparedBenchmark {
    #[instrument(skip(self, context, ids), fields(ids_count = ids.len()))]
    async fn run(
        &self,
        context: &BenchmarkContext,
        ids: &[[u8; 32]],
    ) -> BenchmarkResult<Vec<ExampleData>> {
        let query = build_prepared_query(MAX_VALUES);
        let mut all_overrides = Vec::new();

        for chunk in ids.chunks(MAX_VALUES) {
            // Build the query with the appropriate number of placeholders
            let chunk_query = if chunk.len() < MAX_VALUES {
                build_prepared_query(chunk.len())
            } else {
                query.clone()
            };

            // Bind the IDs to the query
            let mut query_builder = sqlx::query_as(&chunk_query);
            for id in chunk.iter() {
                query_builder = query_builder.bind(*id);
            }

            // Execute the query
            let overrides: Vec<ExampleData> = query_builder
                .fetch_all(&context.pool)
                .await
                .map_err(BenchmarkError::Database)?;

            all_overrides.extend(overrides);
        }

        Ok(all_overrides)
    }

    fn name(&self) -> &'static str {
        "chunked_prepared"
    }

    fn description(&self) -> &'static str {
        "Splits IDs into chunks and uses prepared statements with placeholders"
    }
}

/// Build a prepared query string with the specified number of placeholders
fn build_prepared_query(num_values: usize) -> String {
    let mut query = "SELECT response FROM overrides WHERE hash IN (".to_owned();

    for i in 0..num_values {
        if i > 0 {
            query.push_str(", ");
        }
        query.push_str(&format!("${}", i + 1));
    }

    query.push_str(");");
    query
}
