#!/bin/bash

# Aerugo Development Environment Setup Script
# This script sets up all external dependencies for development
# including Docker containers for PostgreSQL, Redis, and MinIO on non-default ports

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to load environment variables from .env file
load_env_vars() {
    if [ -f ".env" ]; then
        print_step "Loading environment variables from .env file..."
        # Export environment variables from .env file, ignoring comments and empty lines
        export $(grep -v '^#' .env | grep -v '^$' | xargs)
        print_success "Environment variables loaded"
    else
        print_error ".env file not found. Please create one with proper configuration."
        exit 1
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
    else
        print_error "Invalid DATABASE_URL format in .env file"
        exit 1
    fi

    # Parse REDIS_URL to extract port
    if [[ "$REDIS_URL" =~ redis://([^:]+):([0-9]+) ]]; then
        REDIS_HOST="${BASH_REMATCH[1]}"
        REDIS_PORT="${BASH_REMATCH[2]}"
    else
        print_error "Invalid REDIS_URL format in .env file"
        exit 1
    fi

    # Parse S3 configuration
    if [[ "$S3_ENDPOINT" =~ http://([^:]+):([0-9]+) ]]; then
        MINIO_HOST="${BASH_REMATCH[1]}"
        MINIO_PORT="${BASH_REMATCH[2]}"
    else
        print_error "Invalid S3_ENDPOINT format in .env file"
        exit 1
    fi

    # Set MinIO console port (API port + 1)
    MINIO_CONSOLE_PORT=$((MINIO_PORT + 1))
    
    # Use S3 credentials from environment
    MINIO_ACCESS_KEY="$S3_ACCESS_KEY"
    MINIO_SECRET_KEY="$S3_SECRET_KEY"
    MINIO_BUCKET="$S3_BUCKET"
}

# Docker network name
NETWORK_NAME=aerugo-dev-network

print_header() {
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}  Aerugo Development Environment Setup${NC}"
    echo -e "${BLUE}================================================${NC}"
}

print_step() {
    echo -e "${YELLOW}>>> $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

check_docker() {
    print_step "Checking Docker installation..."
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    if ! docker info &> /dev/null; then
        print_error "Docker is not running. Please start Docker first."
        exit 1
    fi
    
    print_success "Docker is installed and running"
}

create_network() {
    print_step "Creating Docker network..."
    if docker network ls | grep -q "$NETWORK_NAME"; then
        print_success "Docker network '$NETWORK_NAME' already exists"
    else
        docker network create "$NETWORK_NAME"
        print_success "Docker network '$NETWORK_NAME' created"
    fi
}

setup_postgres() {
    print_step "Setting up PostgreSQL container..."
    
    # Stop and remove existing container if it exists
    if docker ps -a | grep -q "aerugo-postgres"; then
        docker stop aerugo-postgres || true
        docker rm aerugo-postgres || true
    fi
    
    # Run PostgreSQL container
    docker run -d \
        --name aerugo-postgres \
        --network "$NETWORK_NAME" \
        -p "${POSTGRES_PORT}:5432" \
        -e POSTGRES_DB="$POSTGRES_DB" \
        -e POSTGRES_USER="$POSTGRES_USER" \
        -e POSTGRES_PASSWORD="$POSTGRES_PASSWORD" \
        -e POSTGRES_INITDB_ARGS="--encoding=UTF-8 --lc-collate=C --lc-ctype=C" \
        -v aerugo-postgres-data:/var/lib/postgresql/data \
        postgres:15-alpine
    
    print_success "PostgreSQL container started on port $POSTGRES_PORT"
}

setup_redis() {
    print_step "Setting up Redis container..."
    
    # Stop and remove existing container if it exists
    if docker ps -a | grep -q "aerugo-redis"; then
        docker stop aerugo-redis || true
        docker rm aerugo-redis || true
    fi
    
    # Run Redis container
    docker run -d \
        --name aerugo-redis \
        --network "$NETWORK_NAME" \
        -p "${REDIS_PORT}:6379" \
        -v aerugo-redis-data:/data \
        redis:7-alpine \
        redis-server --appendonly yes
    
    print_success "Redis container started on port $REDIS_PORT"
}

setup_minio() {
    print_step "Setting up MinIO container..."
    
    # Stop and remove existing container if it exists
    if docker ps -a | grep -q "aerugo-minio"; then
        docker stop aerugo-minio || true
        docker rm aerugo-minio || true
    fi
    
    # Run MinIO container
    docker run -d \
        --name aerugo-minio \
        --network "$NETWORK_NAME" \
        -p "${MINIO_PORT}:9000" \
        -p "${MINIO_CONSOLE_PORT}:9001" \
        -e MINIO_ACCESS_KEY="$MINIO_ACCESS_KEY" \
        -e MINIO_SECRET_KEY="$MINIO_SECRET_KEY" \
        -v aerugo-minio-data:/data \
        quay.io/minio/minio:latest \
        server /data --console-address ":9001"
    
    print_success "MinIO container started on port $MINIO_PORT (Console: $MINIO_CONSOLE_PORT)"
}

wait_for_services() {
    print_step "Waiting for services to be ready..."
    
    # Wait for PostgreSQL
    echo -n "Waiting for PostgreSQL..."
    for i in {1..30}; do
        if docker exec aerugo-postgres pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB" &> /dev/null; then
            echo " Ready!"
            break
        fi
        echo -n "."
        sleep 1
    done
    
    # Wait for Redis
    echo -n "Waiting for Redis..."
    for i in {1..30}; do
        if docker exec aerugo-redis redis-cli ping &> /dev/null; then
            echo " Ready!"
            break
        fi
        echo -n "."
        sleep 1
    done
    
    # Wait for MinIO
    echo -n "Waiting for MinIO..."
    for i in {1..30}; do
        if curl -s "http://localhost:$MINIO_PORT/minio/health/ready" &> /dev/null; then
            echo " Ready!"
            break
        fi
        echo -n "."
        sleep 1
    done
}

setup_minio_bucket() {
    print_step "Setting up MinIO bucket..."
    
    # Install mc (MinIO client) if not present
    if ! command -v mc &> /dev/null; then
        print_step "Installing MinIO client..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            curl -L https://dl.min.io/client/mc/release/linux-amd64/mc -o /tmp/mc
            chmod +x /tmp/mc
            sudo mv /tmp/mc /usr/local/bin/mc
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install minio/stable/mc 2>/dev/null || {
                curl -L https://dl.min.io/client/mc/release/darwin-amd64/mc -o /tmp/mc
                chmod +x /tmp/mc
                sudo mv /tmp/mc /usr/local/bin/mc
            }
        fi
    fi
    
    # Configure MinIO client
    mc alias set aerugo-local "http://localhost:$MINIO_PORT" "$MINIO_ACCESS_KEY" "$MINIO_SECRET_KEY" || true
    
    # Create bucket if it doesn't exist
    if ! mc ls aerugo-local | grep -q "$MINIO_BUCKET"; then
        mc mb "aerugo-local/$MINIO_BUCKET"
        print_success "MinIO bucket '$MINIO_BUCKET' created"
    else
        print_success "MinIO bucket '$MINIO_BUCKET' already exists"
    fi
    
    # Set bucket policy to allow public read access (for development)
    mc anonymous set public "aerugo-local/$MINIO_BUCKET"
}

validate_env_file() {
    print_step "Validating .env file configuration..."
    
    # Check if required environment variables are set
    required_vars=("DATABASE_URL" "REDIS_URL" "S3_ENDPOINT" "S3_BUCKET" "S3_ACCESS_KEY" "S3_SECRET_KEY")
    
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var}" ]]; then
            print_error "Required environment variable $var is not set in .env file"
            exit 1
        fi
    done
    
    print_success ".env file configuration is valid"
}

show_connection_info() {
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}  Development Environment Ready!${NC}"
    echo -e "${BLUE}================================================${NC}"
    echo ""
    echo -e "${GREEN}PostgreSQL:${NC}"
    echo -e "  Host: localhost"
    echo -e "  Port: ${POSTGRES_PORT}"
    echo -e "  Database: ${POSTGRES_DB}"
    echo -e "  Username: ${POSTGRES_USER}"
    echo -e "  Password: ${POSTGRES_PASSWORD}"
    echo -e "  Connection: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${POSTGRES_PORT}/${POSTGRES_DB}"
    echo ""
    echo -e "${GREEN}Redis:${NC}"
    echo -e "  Host: localhost"
    echo -e "  Port: ${REDIS_PORT}"
    echo -e "  Connection: redis://localhost:${REDIS_PORT}"
    echo ""
    echo -e "${GREEN}MinIO (S3-compatible):${NC}"
    echo -e "  API Endpoint: http://localhost:${MINIO_PORT}"
    echo -e "  Console: http://localhost:${MINIO_CONSOLE_PORT}"
    echo -e "  Access Key: ${MINIO_ACCESS_KEY}"
    echo -e "  Secret Key: ${MINIO_SECRET_KEY}"
    echo -e "  Bucket: ${MINIO_BUCKET}"
    echo ""
    echo -e "${YELLOW}Management Commands:${NC}"
    echo -e "  View logs: docker logs aerugo-postgres|aerugo-redis|aerugo-minio"
    echo -e "  Stop all: docker stop aerugo-postgres aerugo-redis aerugo-minio"
    echo -e "  Start all: docker start aerugo-postgres aerugo-redis aerugo-minio"
    echo -e "  Remove all: docker rm aerugo-postgres aerugo-redis aerugo-minio"
    echo ""
}

# Main execution
main() {
    print_header
    
    # Load environment variables first
    load_env_vars
    parse_env_vars
    validate_env_file
    
    check_docker
    create_network
    setup_postgres
    setup_redis
    setup_minio
    wait_for_services
    setup_minio_bucket
    show_connection_info
    
    echo -e "${GREEN}🎉 Development environment setup completed successfully!${NC}"
    echo -e "${YELLOW}You can now start developing with 'cargo run'${NC}"
}

# Handle script arguments
case "${1:-setup}" in
    "setup")
        main
        ;;
    "stop")
        load_env_vars
        parse_env_vars
        print_step "Stopping all containers..."
        docker stop aerugo-postgres aerugo-redis aerugo-minio 2>/dev/null || true
        print_success "All containers stopped"
        ;;
    "start")
        load_env_vars
        parse_env_vars
        print_step "Starting all containers..."
        docker start aerugo-postgres aerugo-redis aerugo-minio 2>/dev/null || true
        print_success "All containers started"
        ;;
    "clean")
        print_step "Cleaning up all containers and volumes..."
        docker stop aerugo-postgres aerugo-redis aerugo-minio 2>/dev/null || true
        docker rm aerugo-postgres aerugo-redis aerugo-minio 2>/dev/null || true
        docker volume rm aerugo-postgres-data aerugo-redis-data aerugo-minio-data 2>/dev/null || true
        docker network rm "$NETWORK_NAME" 2>/dev/null || true
        print_success "Cleanup completed"
        ;;
    "status")
        print_step "Checking container status..."
        echo -e "${GREEN}PostgreSQL:${NC} $(docker ps --filter name=aerugo-postgres --format 'table {{.Status}}')"
        echo -e "${GREEN}Redis:${NC} $(docker ps --filter name=aerugo-redis --format 'table {{.Status}}')"
        echo -e "${GREEN}MinIO:${NC} $(docker ps --filter name=aerugo-minio --format 'table {{.Status}}')"
        ;;
    *)
        echo "Usage: $0 [setup|stop|start|clean|status]"
        echo "  setup  - Set up the development environment (default)"
        echo "  stop   - Stop all containers"
        echo "  start  - Start all containers"
        echo "  clean  - Remove all containers and volumes"
        echo "  status - Show container status"
        exit 1
        ;;
esac
