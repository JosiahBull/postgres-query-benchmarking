use crate::{BenchmarkContext, BenchmarkResult, BenchmarkTest, ExampleData};
use async_trait::async_trait;
use tracing::instrument;

/// Benchmark that uses temporary table with text COPY
pub struct TempTableTextCopyBenchmark;

#[async_trait]
impl BenchmarkTest for TempTableTextCopyBenchmark {
    #[instrument(skip(self, context, ids), fields(ids_count = ids.len()))]
    async fn run(
        &self,
        context: &BenchmarkContext,
        ids: &[i64],
    ) -> BenchmarkResult<Vec<ExampleData>> {
        let mut transaction = context.pool.begin().await?;

        // Create a temporary unlogged table to hold the IDs
        sqlx::query("CREATE UNLOGGED TABLE temp_ids (id BIGINT PRIMARY KEY);")
            .execute(&mut *transaction)
            .await?;

        // Get a copy-in handle for the temporary table
        let mut handle = transaction
            .copy_in_raw("COPY temp_ids (id) FROM STDIN")
            .await?;

        // Prepare the IDs as text format with newlines
        let ids_as_bytes: Vec<u8> = ids.iter().map(|id| id.to_string().into_bytes()).fold(
            Vec::with_capacity(ids.len() * 20), //estimate...
            |mut acc, id_bytes| {
                if !acc.is_empty() {
                    acc.push(b'\n'); // Add newline between IDs
                }
                acc.extend(id_bytes);
                acc
            },
        );

        // Send the data to PostgreSQL
        handle.send(ids_as_bytes).await?;
        handle.finish().await?;

        // Perform the query using the temporary table
        let result: Vec<ExampleData> = sqlx::query_as(
            "SELECT RESPONSE as response FROM OVERRIDES WHERE HASH IN (SELECT id FROM temp_ids);",
        )
        .fetch_all(&mut *transaction)
        .await?;

        // Rollback to clean up the temporary table
        transaction.rollback().await?;

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "temp_table_text_copy"
    }

    fn description(&self) -> &'static str {
        "Creates temporary table and uses COPY with text format"
    }

    async fn cleanup(&self, context: &BenchmarkContext) -> BenchmarkResult<()> {
        // Clear caches after the benchmark
        context.clear_caches().await?;

        // Clean up any leftover temp tables (just in case)
        let _ = sqlx::query("DROP TABLE IF EXISTS temp_ids;")
            .execute(&context.pool)
            .await;

        Ok(())
    }
}
