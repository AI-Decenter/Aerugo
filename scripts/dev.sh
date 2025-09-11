#!/bin/bash

# Aerugo Development Helper Script
# This script provides common development tasks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Function to load environment variables from .env file
load_env_vars() {
    if [ -f ".env" ]; then
        # Export environment variables from .env file, ignoring comments and empty lines
        export $(grep -v '^#' .env | grep -v '^$' | xargs)
    fi
}

# Function to extract values from environment variables
parse_env_vars() {
    # Parse DATABASE_URL to extract components
    if [[ "$DATABASE_URL" =~ postgresql://([^:]+):([^@]+)@([^:]+):([0-9]+)/(.+) ]]; then
        POSTGRES_USER="${BASH_REMATCH[1]}"
        POSTGRES_PASSWORD="${BASH_REMATCH[2]}"
        POSTGRES_HOST="${BASH_REMATCH[3]}"
        POSTGRES_PORT="${BASH_REMATCH[4]}"
        POSTGRES_DB="${BASH_REMATCH[5]}"
    fi

    # Parse REDIS_URL to extract port
    if [[ "$REDIS_URL" =~ redis://([^:]+):([0-9]+) ]]; then
        REDIS_HOST="${BASH_REMATCH[1]}"
        REDIS_PORT="${BASH_REMATCH[2]}"
    fi

    # Parse S3 configuration
    if [[ "$S3_ENDPOINT" =~ http://([^:]+):([0-9]+) ]]; then
        MINIO_HOST="${BASH_REMATCH[1]}"
        MINIO_PORT="${BASH_REMATCH[2]}"
    fi

    # Set MinIO console port (API port + 1)
    MINIO_CONSOLE_PORT=$((MINIO_PORT + 1))
}

# Load environment variables at the start
load_env_vars
parse_env_vars

print_step() {
    echo -e "${YELLOW}>>> $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Function to show help
show_help() {
    echo -e "${BLUE}Aerugo Development Helper${NC}"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  setup       - Set up development environment"
    echo "  start       - Start all development services"
    echo "  stop        - Stop all development services"
    echo "  restart     - Restart all development services"
    echo "  status      - Show service status"
    echo "  logs        - Show logs for all services"
    echo "  psql        - Connect to PostgreSQL database"
    echo "  redis-cli   - Connect to Redis"
    echo "  minio       - Open MinIO console"
    echo "  clean       - Clean up all containers and volumes"
    echo "  build       - Build the Rust application"
    echo "  run         - Run the Rust application in development mode"
    echo "  test        - Run tests"
    echo "  fmt         - Format code"
    echo "  check       - Check code without building"
    echo ""
}

# Function to run the main setup script
setup() {
    print_step "Running development environment setup..."
    ./scripts/setup-dev-env.sh setup
}

# Function to start services
start() {
    print_step "Starting development services..."
    ./scripts/setup-dev-env.sh start
}

# Function to stop services
stop() {
    print_step "Stopping development services..."
    ./scripts/setup-dev-env.sh stop
}

# Function to restart services
restart() {
    print_step "Restarting development services..."
    ./scripts/setup-dev-env.sh stop
    sleep 2
    ./scripts/setup-dev-env.sh start
}

# Function to show status
status() {
    ./scripts/setup-dev-env.sh status
}

# Function to show logs
show_logs() {
    print_step "Showing logs for all services..."
    echo -e "${GREEN}PostgreSQL logs:${NC}"
    docker logs --tail=20 aerugo-postgres 2>/dev/null || echo "PostgreSQL container not running"
    echo ""
    echo -e "${GREEN}Redis logs:${NC}"
    docker logs --tail=20 aerugo-redis 2>/dev/null || echo "Redis container not running"
    echo ""
    echo -e "${GREEN}MinIO logs:${NC}"
    docker logs --tail=20 aerugo-minio 2>/dev/null || echo "MinIO container not running"
}

# Function to connect to PostgreSQL
connect_psql() {
    print_step "Connecting to PostgreSQL..."
    docker exec -it aerugo-postgres psql -U "$POSTGRES_USER" -d "$POSTGRES_DB"
}

# Function to connect to Redis
connect_redis() {
    print_step "Connecting to Redis..."
    docker exec -it aerugo-redis redis-cli
}

# Function to open MinIO console
open_minio() {
    print_step "Opening MinIO console..."
    echo -e "${GREEN}MinIO Console: http://localhost:$MINIO_CONSOLE_PORT${NC}"
    echo -e "${GREEN}Access Key: $S3_ACCESS_KEY${NC}"
    echo -e "${GREEN}Secret Key: $S3_SECRET_KEY${NC}"
    
    # Try to open browser if available
    if command -v xdg-open &> /dev/null; then
        xdg-open "http://localhost:$MINIO_CONSOLE_PORT"
    elif command -v open &> /dev/null; then
        open "http://localhost:$MINIO_CONSOLE_PORT"
    else
        echo -e "${YELLOW}Please open http://localhost:$MINIO_CONSOLE_PORT in your browser${NC}"
    fi
}

# Function to clean up
clean() {
    print_step "Cleaning up development environment..."
    ./scripts/setup-dev-env.sh clean
}

# Rust development functions
build() {
    print_step "Building Rust application..."
    # Load environment variables for build
    load_env_vars
    cargo build
    print_success "Build completed"
}

run_app() {
    print_step "Running Rust application in development mode..."
    # Load environment variables for runtime
    load_env_vars
    cargo run
}

test() {
    print_step "Running tests..."
    # Load environment variables for tests
    load_env_vars
    cargo test
}

fmt() {
    print_step "Formatting code..."
    cargo fmt
    print_success "Code formatted"
}

check() {
    print_step "Checking code..."
    cargo check
    print_success "Code check completed"
}

# Main execution
case "${1:-help}" in
    "setup")
        setup
        ;;
    "start")
        start
        ;;
    "stop")
        stop
        ;;
    "restart")
        restart
        ;;
    "status")
        status
        ;;
    "logs")
        show_logs
        ;;
    "psql")
        connect_psql
        ;;
    "redis-cli")
        connect_redis
        ;;
    "minio")
        open_minio
        ;;
    "clean")
        clean
        ;;
    "build")
        build
        ;;
    "run")
        run_app
        ;;
    "test")
        test
        ;;
    "fmt")
        fmt
        ;;
    "check")
        check
        ;;
    "help"|*)
        show_help
        ;;
esac
