-- PostgreSQL Cold Query Configuration Script
-- This script configures PostgreSQL for consistent cold query performance testing
-- by disabling JIT compilation and other optimizations that could affect benchmarking

-- Disable JIT compilation entirely
ALTER SYSTEM SET jit = 'off';

-- Force custom plans to avoid plan caching effects
ALTER SYSTEM SET plan_cache_mode = 'force_custom_plan';

-- Disable automatic statistics collection during benchmarks
ALTER SYSTEM SET autovacuum = 'off';
ALTER SYSTEM SET track_activities = 'off';
ALTER SYSTEM SET track_counts = 'off';
ALTER SYSTEM SET track_io_timing = 'off';

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
