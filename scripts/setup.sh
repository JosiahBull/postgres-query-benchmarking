#!/usr/bin/env bash

# PostgreSQL Benchmark Setup Script
# Simplified version that manages Docker containers for benchmarking

set -euo pipefail

# Configuration
CONTAINER_NAME="pg-benchmark"
POSTGRES_VERSION="16"
POSTGRES_USER="postgres"
POSTGRES_PASSWORD="postgres"
POSTGRES_DB="postgres"
POSTGRES_PORT="5432"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DATA_DIR="$PROJECT_DIR/data"

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
        rm -rf "$DATA_DIR"
        log_success "Data directory removed"
    fi
}

setup_container() {
    log_info "Setting up PostgreSQL container for benchmarking..."

    check_docker

    # Create data directory
    mkdir -p "$DATA_DIR"

    # Create and start container with optimized settings for benchmarking
    log_info "Creating PostgreSQL container with benchmark optimizations..."
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

    local sql_script2="$SCRIPT_DIR/cold_query_init.sql"
    if [[ -f "$sql_script2" ]]; then
        log_info "Running cold query initialization script..."
        docker exec -i "$CONTAINER_NAME" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" < "$sql_script2"
        log_success "Cold query initialization completed"
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

    log_success "PostgreSQL container setup completed"
    log_info "Connection URL: postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:$POSTGRES_PORT/$POSTGRES_DB"
}

main() {
    cleanup_existing
    setup_container
}

main "$@"
