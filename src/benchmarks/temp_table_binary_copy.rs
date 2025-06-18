use crate::{BenchmarkContext, BenchmarkResult, BenchmarkTest, ExampleData};
use async_trait::async_trait;
use tracing::instrument;

/// Benchmark that uses temporary table with binary COPY
pub struct TempTableBinaryCopyBenchmark;

#[async_trait]
impl BenchmarkTest for TempTableBinaryCopyBenchmark {
    #[instrument(skip(self, context, ids), fields(ids_count = ids.len()))]
    async fn run(
        &self,
        context: &BenchmarkContext,
        ids: &[[u8; 32]],
    ) -> BenchmarkResult<Vec<ExampleData>> {
        let mut transaction = context.pool.begin().await?;

        // Create a temporary unlogged table to hold the IDs
        sqlx::query("CREATE UNLOGGED TABLE temp_ids (id BYTEA PRIMARY KEY);")
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

        // Send the binary format header
        handle.send(&SIG[..]).await?;

        // Binary format structure constants
        const LENGTH_PER_FIELD: u32 = std::mem::size_of::<[u8; 32]>() as u32;
        const SIZE_PER_TUPLE: usize =
            std::mem::size_of::<i16>() + std::mem::size_of::<u32>() + LENGTH_PER_FIELD as usize;
        const NUM_FIELDS_PER_TUPLE: i16 = 1;

        // Process IDs in chunks to manage memory usage
        const MAX_CHUNK_SIZE: usize = 4096;
        const ACTUAL_CHUNK_SIZE: usize = {
            let mut size = MAX_CHUNK_SIZE;
            while size % SIZE_PER_TUPLE != 0 {
                size -= 1;
            }
            size
        };

        // Pre-fill buffer with tuple structure (optimization)
        let mut buf: [u8; ACTUAL_CHUNK_SIZE] = {
            let mut init = [0; ACTUAL_CHUNK_SIZE];
            let mut offset = 0;
            while offset < ACTUAL_CHUNK_SIZE {
                // Number of fields in tuple
                init[offset..offset + std::mem::size_of::<i16>()]
                    .copy_from_slice(&NUM_FIELDS_PER_TUPLE.to_be_bytes());
                offset += std::mem::size_of::<i16>();
                // Length of field
                init[offset..offset + std::mem::size_of::<u32>()]
                    .copy_from_slice(&LENGTH_PER_FIELD.to_be_bytes());
                offset += std::mem::size_of::<u32>();
                // Skip the ID value (will be filled per chunk)
                offset += LENGTH_PER_FIELD as usize;
            }
            init
        };

        // Send IDs in chunks
        for chunk in ids.chunks(ACTUAL_CHUNK_SIZE / SIZE_PER_TUPLE) {
            let mut offset = 0;
            for id in chunk.iter() {
                // Fill in the ID value in the pre-structured buffer
                buf[offset + std::mem::size_of::<i16>() + std::mem::size_of::<u32>()
                    ..offset + SIZE_PER_TUPLE]
                    .copy_from_slice(id);
                offset += SIZE_PER_TUPLE;
            }
            // Send the buffer with actual data
            handle.send(&buf[..offset]).await?;
        }

        // Finish the COPY operation
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
        "temp_table_binary_copy"
    }

    fn description(&self) -> &'static str {
        "Creates temporary table and uses COPY with binary format"
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
