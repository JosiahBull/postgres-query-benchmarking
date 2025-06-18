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
        ids: &[[u8; 32]],
    ) -> BenchmarkResult<Vec<ExampleData>> {
        // Couldn't get this to work. :(
        return Err(crate::BenchmarkError::Setup {
            message: "COPY with text format not supported in this benchmark".to_string(),
        });

        let mut transaction = context.pool.begin().await?;

        // Create a temporary unlogged table to hold the IDs
        sqlx::query("CREATE UNLOGGED TABLE temp_ids (id BYTEA PRIMARY KEY);")
            .execute(&mut *transaction)
            .await?;

        // Get a copy-in handle for the temporary table
        let mut handle = transaction
            .copy_in_raw("COPY temp_ids (id) FROM STDIN")
            .await?;

        // Prepare the IDs as text format with newlines
        let ids_as_bytes: Vec<u8> = ids
            .iter()
            .map(|id| {
                // Reverse the byte order to get big-endian encoding
                let id_be: Vec<u8> = id.iter().rev().copied().collect();
                let id_hex = hex::encode(hex::encode(id_be).to_uppercase()).to_uppercase();
                format!("\\x{}\n", id_hex)
            })
            .fold(
                Vec::with_capacity(ids.len() * 30), //estimate...
                |mut acc, id_str| {
                    acc.extend(id_str.as_bytes());
                    acc
                },
            );

        // Send the data to PostgreSQL
        handle.send(ids_as_bytes).await?;
        handle.finish().await?;

        // Perform the query using the temporary table
        let result: Vec<ExampleData> = sqlx::query_as(
            "SELECT response FROM overrides WHERE hash IN (SELECT id FROM temp_ids);",
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
