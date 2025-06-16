-- PostgreSQL Cold Query Configuration Script
-- This script configures PostgreSQL for consistent cold query performance testing
-- by disabling JIT compilation and other optimizations that could affect benchmarking

-- Disable JIT compilation entirely
ALTER SYSTEM SET jit = 'off';
ALTER SYSTEM SET jit_above_cost = -1;
ALTER SYSTEM SET jit_inline_above_cost = -1;
ALTER SYSTEM SET jit_optimize_above_cost = -1;

-- Force custom plans to avoid plan caching effects
ALTER SYSTEM SET plan_cache_mode = 'force_custom_plan';

-- Set conservative cache and cost settings for consistent performance
ALTER SYSTEM SET effective_cache_size = '128MB';
ALTER SYSTEM SET random_page_cost = 4.0;
ALTER SYSTEM SET seq_page_cost = 1.0;
ALTER SYSTEM SET cpu_tuple_cost = 0.01;
ALTER SYSTEM SET cpu_index_tuple_cost = 0.005;
ALTER SYSTEM SET cpu_operator_cost = 0.0025;

-- Enable detailed logging for query analysis
ALTER SYSTEM SET log_statement = 'all';
ALTER SYSTEM SET log_duration = 'on';
ALTER SYSTEM SET log_min_duration_statement = 0;

-- Set reasonable connection limits
ALTER SYSTEM SET max_connections = 100;

-- Disable automatic statistics collection during benchmarks
ALTER SYSTEM SET autovacuum = 'off';
ALTER SYSTEM SET track_activities = 'off';
ALTER SYSTEM SET track_counts = 'off';

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
    'cpu_tuple_cost',
    'cpu_index_tuple_cost',
    'cpu_operator_cost',
    'log_statement',
    'log_duration',
    'max_connections',
    'autovacuum'
)
ORDER BY name;
