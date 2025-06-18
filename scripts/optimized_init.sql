-- PostgreSQL Optimized Configuration Script
-- This script configures PostgreSQL for maximum performance testing
-- by enabling JIT compilation and other optimizations

-- Enable JIT compilation for complex queries
ALTER SYSTEM SET jit = 'on';

-- Enable plan caching for better performance
ALTER SYSTEM SET plan_cache_mode = 'auto';

-- Set aggressive cache settings for performance
ALTER SYSTEM SET effective_cache_size = '2GB';

-- Enable parallel query execution
ALTER SYSTEM SET max_parallel_workers_per_gather = 4;

-- Optimize memory settings
ALTER SYSTEM SET work_mem = '64MB';
ALTER SYSTEM SET maintenance_work_mem = '256MB';
ALTER SYSTEM SET shared_buffers = '256MB';

-- Optimize checkpoint and WAL settings
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '32MB';
ALTER SYSTEM SET min_wal_size = '1GB';
ALTER SYSTEM SET max_wal_size = '4GB';

-- Disable autovacuum and statistics collection
ALTER SYSTEM SET autovacuum = 'off';
ALTER SYSTEM SET track_activities = 'off';
ALTER SYSTEM SET track_counts = 'off';
ALTER SYSTEM SET track_io_timing = 'off';

-- Disable excessive logging for performance
ALTER SYSTEM SET log_statement = 'none';
ALTER SYSTEM SET log_duration = 'off';
ALTER SYSTEM SET log_min_duration_statement = -1;

-- Reload configuration to apply changes
SELECT pg_reload_conf();

-- Show applied settings for verification
SELECT name, setting, context, pending_restart
FROM pg_settings
WHERE name IN (
    'jit',
    'jit_above_cost',
    'jit_inline_above_cost',
    'jit_optimize_above_cost',
    'plan_cache_mode',
    'effective_cache_size',
    'random_page_cost',
    'seq_page_cost',
    'work_mem',
    'maintenance_work_mem',
    'shared_buffers',
    'max_parallel_workers_per_gather',
    'max_parallel_workers',
    'autovacuum',
    'track_activities',
    'track_counts',
    'default_statistics_target'
)
ORDER BY name;
