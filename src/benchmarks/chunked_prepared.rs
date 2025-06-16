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
        ids: &[i64],
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
                .map_err(|e| BenchmarkError::Database(e))?;

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
    let mut query = "SELECT RESPONSE as response FROM OVERRIDES WHERE HASH IN (".to_owned();

    for i in 0..num_values {
        if i > 0 {
            query.push_str(", ");
        }
        query.push_str(&format!("${}", i + 1));
    }

    query.push_str(");");
    query
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prepared_query() {
        let query = build_prepared_query(3);
        assert_eq!(
            query,
            "SELECT RESPONSE as response FROM OVERRIDES WHERE HASH IN ($1, $2, $3);"
        );
    }

    #[test]
    fn test_build_prepared_query_single() {
        let query = build_prepared_query(1);
        assert_eq!(
            query,
            "SELECT RESPONSE as response FROM OVERRIDES WHERE HASH IN ($1);"
        );
    }
}
