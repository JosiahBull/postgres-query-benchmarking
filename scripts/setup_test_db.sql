-- Add the pgcrypto extension for hashing
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Create the OVERRIDES table
DROP TABLE IF EXISTS overrides;
CREATE TABLE overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hash BYTEA NOT NULL UNIQUE,
    response TEXT NOT NULL
);

-- Create index for performance (crucial for benchmarks)
CREATE INDEX idx_overrides_hash ON overrides (hash) INCLUDE (response);

-- Insert sample data (20 million rows) using batched approach for better performance
DO $$
DECLARE
    batch_size INTEGER := 1000000; -- 1 million rows per batch
    total_rows INTEGER := 20000000;
    current_start INTEGER := 1;
    current_end INTEGER;
BEGIN
    -- Temporarily disable autovacuum and increase work_mem for this session
    -- PERFORM set_config('autovacuum', 'off', true);
    PERFORM set_config('work_mem', '1GB', true);

    WHILE current_start <= total_rows LOOP
        current_end := LEAST(current_start + batch_size - 1, total_rows);

        INSERT INTO overrides (hash, response)
        SELECT
            -- Hash the input with SHA256 returning bytea
            digest(gs::text, 'sha256') AS hash,
            -- Generate a random 20-character response
            substring(md5(random()::text) from 1 for 20) AS response
        FROM generate_series(current_start, current_end) AS gs;

        -- Log progress every 10 million rows
        IF current_start % 10000000 = 1 THEN
            RAISE NOTICE 'Inserted % rows so far', current_end;
        END IF;

        current_start := current_end + 1;
    END LOOP;
END $$;

-- Analyze table for better query planning
VACUUM ANALYZE overrides;

-- Display table statistics
SELECT
    schemaname,
    tablename,
    attname,
    n_distinct,
    correlation
FROM
    pg_stats
WHERE
    tablename = 'overrides';

-- Show index information
SELECT
    indexname,
    indexdef
FROM
    pg_indexes
WHERE
    tablename = 'overrides';
