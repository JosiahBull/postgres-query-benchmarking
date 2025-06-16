-- PostgreSQL Test Database Setup Script
-- Create the OVERRIDES table
DROP TABLE IF EXISTS OVERRIDES;

CREATE TABLE OVERRIDES (
    ID UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    HASH BIGINT NOT NULL UNIQUE,
    RESPONSE TEXT NOT NULL
);

-- Create index for performance (crucial for benchmarks)
CREATE INDEX idx_overrides_hash ON OVERRIDES (HASH) INCLUDE (RESPONSE);

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

        INSERT INTO OVERRIDES (HASH, RESPONSE)
        SELECT
            -- Create a fake hash that counts up serially
            gs AS HASH,
            -- Create a fake response that is a simple string concatenation
            'Response for hash ' || gs AS RESPONSE
        FROM generate_series(current_start, current_end) AS gs;

        -- Log progress every 10 million rows
        IF current_start % 10000000 = 1 THEN
            RAISE NOTICE 'Inserted % rows so far', current_end;
        END IF;

        current_start := current_end + 1;
    END LOOP;
END $$;

-- Analyze table for better query planning
ANALYZE OVERRIDES;

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

VACUUM ANALYZE OVERRIDES;
