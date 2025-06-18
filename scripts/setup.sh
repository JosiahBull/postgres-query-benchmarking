#!/usr/bin/env bash

# PostgreSQL Benchmark Setup Script
# Manages Docker containers for benchmarking with optimization modes

set -euo pipefail

# Configuration
CONTAINER_NAME="pg-benchmark"
POSTGRES_VERSION="latest"
POSTGRES_USER="postgres"
POSTGRES_PASSWORD="postgres"
POSTGRES_DB="postgres"
POSTGRES_PORT="5432"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DATA_DIR="$PROJECT_DIR/data"

# Default mode is cold (for benchmarking)
MODE="cold"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -m, --mode MODE    Set PostgreSQL optimization mode:"
    echo "                     cold     - Cold query mode (default, for benchmarking)"
    echo "                     optimized - Optimized mode (for performance testing)"
    echo "  -h, --help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run with cold query mode (default)"
    echo "  $0 -m cold          # Explicitly set cold query mode"
    echo "  $0 -m optimized     # Run with optimized settings"
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -m|--mode)
                MODE="$2"
                if [[ "$MODE" != "cold" && "$MODE" != "optimized" ]]; then
                    log_error "Invalid mode: $MODE. Must be 'cold' or 'optimized'"
                    show_usage
                    exit 1
                fi
                shift 2
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker."
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running. Please start Docker."
        exit 1
    fi
}

container_exists() {
    docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$" 2>/dev/null
}

container_running() {
    docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$" 2>/dev/null
}

cleanup_existing() {
    log_info "Cleaning up existing container and data..."

    check_docker

    # Stop and remove container if it exists
    if container_exists; then
        if container_running; then
            log_info "Stopping existing container..."
            docker stop "$CONTAINER_NAME" &> /dev/null
        fi
        log_info "Removing existing container..."
        docker rm "$CONTAINER_NAME" &> /dev/null
        log_success "Container removed"
    fi

    # Remove data directory if it exists
    if [[ -d "$DATA_DIR" ]]; then
        log_info "Removing existing data directory..."
        sudo rm -rf "$DATA_DIR"
        log_success "Data directory removed"
    fi
}

setup_container() {
    log_info "Setting up PostgreSQL container for benchmarking in $MODE mode..."

    check_docker

    # Create data directory
    mkdir -p "$DATA_DIR"

    # Create and start container
    log_info "Creating PostgreSQL container with cold query settings for benchmarking..."
    docker run -d \
        --name "$CONTAINER_NAME" \
        -e POSTGRES_USER="$POSTGRES_USER" \
        -e POSTGRES_PASSWORD="$POSTGRES_PASSWORD" \
        -e POSTGRES_DB="$POSTGRES_DB" \
        -p "${POSTGRES_PORT}:5432" \
        -v "${DATA_DIR}:/var/lib/postgresql/data" \
        postgres:${POSTGRES_VERSION}

    # Wait for PostgreSQL to be ready
    log_info "Waiting for PostgreSQL to be ready..."
    local max_attempts=30
    local attempt=1

    while [ $attempt -le $max_attempts ]; do
        if docker exec "$CONTAINER_NAME" pg_isready -U "$POSTGRES_USER" &> /dev/null; then
            log_success "PostgreSQL is ready!"
            break
        fi

        if [ $attempt -eq $max_attempts ]; then
            log_error "PostgreSQL failed to start within $max_attempts seconds"
            docker logs "$CONTAINER_NAME"
            exit 1
        fi

        sleep 1
        ((attempt++))
    done

    # Run setup SQL if it exists
    local sql_script="$SCRIPT_DIR/setup_test_db.sql"
    if [[ -f "$sql_script" ]]; then
        log_info "Running database setup script..."
        docker exec -i "$CONTAINER_NAME" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" < "$sql_script"
        log_success "Database setup completed"
    fi

    # Apply mode-specific configuration
    if [[ "$MODE" == "cold" ]]; then
        local cold_script="$SCRIPT_DIR/cold_query_init.sql"
        log_info "Running cold query initialization script..."
        docker exec -i "$CONTAINER_NAME" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" < "$cold_script"
        log_success "Cold query initialization completed"
    else
        local optimized_script="$SCRIPT_DIR/optimized_init.sql"
        log_info "Running optimized configuration script..."
        docker exec -i "$CONTAINER_NAME" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" < "$optimized_script"
        log_success "Optimized configuration completed"
    fi

    # Restart container to apply settings
    log_info "Restarting PostgreSQL container to apply settings..."
    docker restart "$CONTAINER_NAME"
    sleep 5  # Wait for the container to restart

    # Wait for PostgreSQL to be ready again
    log_info "Waiting for PostgreSQL to be ready after restart..."
    attempt=1
    while [ $attempt -le $max_attempts ]; do
        if docker exec "$CONTAINER_NAME" pg_isready -U "$POSTGRES_USER" &> /dev/null; then
            log_success "PostgreSQL is ready after restart!"
            break
        fi

        if [ $attempt -eq $max_attempts ]; then
            log_error "PostgreSQL failed to start after restart within $max_attempts seconds"
            docker logs "$CONTAINER_NAME"
            exit 1
        fi

        sleep 1
        ((attempt++))
    done

    log_success "PostgreSQL container setup completed in $MODE mode"
    log_info "Connection URL: postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:$POSTGRES_PORT/$POSTGRES_DB"

    if [[ "$MODE" == "optimized" ]]; then
        log_info "Running with performance optimizations enabled"
        log_info "JIT compilation, plan caching, and autovacuum are enabled"
    else
        log_info "Running in cold query mode for consistent benchmarking"
        log_info "JIT compilation, plan caching, and autovacuum are disabled"
    fi
}

main() {
    parse_args "$@"
    cleanup_existing
    setup_container
}

main "$@"
