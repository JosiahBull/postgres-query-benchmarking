-- PostgreSQL Optimized Configuration Script
-- This script configures PostgreSQL for maximum performance testing
-- by enabling JIT compilation and other optimizations

-- Enable JIT compilation for complex queries
ALTER SYSTEM SET jit = 'on';
ALTER SYSTEM SET jit_above_cost = 100000;
ALTER SYSTEM SET jit_inline_above_cost = 500000;
ALTER SYSTEM SET jit_optimize_above_cost = 500000;

-- Enable plan caching for better performance
ALTER SYSTEM SET plan_cache_mode = 'auto';

-- Set aggressive cache and cost settings for performance
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET seq_page_cost = 1.0;
ALTER SYSTEM SET cpu_tuple_cost = 0.01;
ALTER SYSTEM SET cpu_index_tuple_cost = 0.005;
ALTER SYSTEM SET cpu_operator_cost = 0.0025;

-- Enable parallel query execution
ALTER SYSTEM SET max_parallel_workers_per_gather = 4;
ALTER SYSTEM SET max_parallel_workers = 8;
ALTER SYSTEM SET parallel_tuple_cost = 0.1;
ALTER SYSTEM SET parallel_setup_cost = 1000.0;

-- Optimize memory settings
ALTER SYSTEM SET work_mem = '16MB';
ALTER SYSTEM SET maintenance_work_mem = '256MB';
ALTER SYSTEM SET shared_buffers = '256MB';

-- Optimize checkpoint and WAL settings
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET min_wal_size = '1GB';
ALTER SYSTEM SET max_wal_size = '4GB';

-- Enable autovacuum and statistics collection
ALTER SYSTEM SET autovacuum = 'on';
ALTER SYSTEM SET autovacuum_max_workers = 3;
ALTER SYSTEM SET autovacuum_naptime = '1min';
ALTER SYSTEM SET track_activities = 'on';
ALTER SYSTEM SET track_counts = 'on';
ALTER SYSTEM SET track_io_timing = 'on';

-- Optimize query planning
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET from_collapse_limit = 8;
ALTER SYSTEM SET join_collapse_limit = 8;

-- Disable excessive logging for performance
ALTER SYSTEM SET log_statement = 'none';
ALTER SYSTEM SET log_duration = 'off';
ALTER SYSTEM SET log_min_duration_statement = -1;

-- Set reasonable connection limits
ALTER SYSTEM SET max_connections = 100;

-- Enable aggressive query optimization
ALTER SYSTEM SET enable_hashjoin = 'on';
ALTER SYSTEM SET enable_mergejoin = 'on';
ALTER SYSTEM SET enable_nestloop = 'on';
ALTER SYSTEM SET enable_seqscan = 'on';
ALTER SYSTEM SET enable_indexscan = 'on';
ALTER SYSTEM SET enable_indexonlyscan = 'on';
ALTER SYSTEM SET enable_bitmapscan = 'on';

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
