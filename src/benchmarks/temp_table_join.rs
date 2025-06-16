use crate::{BenchmarkContext, BenchmarkResult, BenchmarkTest, ExampleData};
use async_trait::async_trait;

/// Benchmark that uses temporary table with JOIN instead of IN
pub struct TempTableJoinBenchmark;

#[async_trait]
impl BenchmarkTest for TempTableJoinBenchmark {
    async fn run(
        &self,
        context: &BenchmarkContext,
        ids: &[i64],
    ) -> BenchmarkResult<Vec<ExampleData>> {
        let mut transaction = context.pool.begin().await?;

        // Create optimized unlogged table with PLAIN storage
        sqlx::query("CREATE UNLOGGED TABLE temp_ids (id BIGINT STORAGE PLAIN PRIMARY KEY);")
            .execute(&mut *transaction)
            .await?;

        // Get a copy-in handle for the temporary table with binary format
        let mut handle = transaction
            .copy_in_raw("COPY temp_ids (id) FROM STDIN WITH (FORMAT BINARY)")
            .await?;

        // PostgreSQL binary format constants
        const SIG: [u8; 19] = [
            b'P', b'G', b'C', b'O', b'P', b'Y', b'\n', 0xFF, b'\r', b'\n', b'\0', 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        // Binary format structure constants
        const SIZE_PER_TUPLE: usize =
            std::mem::size_of::<i16>() + std::mem::size_of::<u32>() + std::mem::size_of::<i64>();
        const NUM_FIELDS_PER_TUPLE: i16 = 1;
        const LENGTH_PER_FIELD: u32 = std::mem::size_of::<i64>() as u32;

        // Pre-allocate buffer with all data at once for optimal performance
        let mut buf: Vec<u8> = Vec::with_capacity(
            (ids.len() * SIZE_PER_TUPLE) + std::mem::size_of::<i16>() + SIG.len(),
        );

        // Add binary format header
        buf.extend_from_slice(&SIG);

        // Add all tuples to buffer
        for id in ids.iter() {
            buf.extend_from_slice(&NUM_FIELDS_PER_TUPLE.to_be_bytes());
            buf.extend_from_slice(&LENGTH_PER_FIELD.to_be_bytes());
            buf.extend_from_slice(&id.to_be_bytes());
        }

        // Add end-of-data marker
        buf.extend_from_slice(&(-1i16).to_be_bytes());

        // Send all data in one operation
        handle.send(buf).await?;
        handle.finish().await?;

        // Use JOIN instead of IN for potentially better performance
        let result: Vec<ExampleData> = sqlx::query_as(
            "SELECT OVERRIDES.RESPONSE as response FROM OVERRIDES JOIN temp_ids ON OVERRIDES.HASH = temp_ids.id;",
        )
        .fetch_all(&mut *transaction)
        .await?;

        // Rollback to clean up the temporary table
        transaction.rollback().await?;

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "temp_table_join"
    }

    fn description(&self) -> &'static str {
        "Creates temporary table with binary COPY and uses JOIN instead of IN clause"
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
