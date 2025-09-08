#!/bin/bash
set -euo pipefail

# Database Constraint Audit Script
# ===============================
#
# This script audits your database for constraints that exist in Postgres
# but weren't created by Drizzle migrations (schema drift detection).
#
# Usage:
#   ./scripts/audit-db-constraints.sh [options]
#
# Options:
#   --help          Show this help message
#   --baseline      Create/update baseline constraint file (db_constraints.txt)
#   --strict        Exit with error if any unknown constraints found
#   --quiet         Reduce output verbosity
#
# Environment:
#   DATABASE_URL    Required: Postgres connection string
#
# Examples:
#   ./scripts/audit-db-constraints.sh
#   ./scripts/audit-db-constraints.sh --baseline
#   ./scripts/audit-db-constraints.sh --strict

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default settings
BASELINE=false
STRICT=false
QUIET=false
MIGRATIONS_DIR="src/nextjs/src/db/migrations"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help)
            cat << 'EOF'
Database Constraint Audit Script
==============================

This script audits your database for constraints that exist in Postgres
but weren't created by Drizzle migrations (schema drift detection).

Usage:
  ./scripts/audit-db-constraints.sh [options]

Options:
  --help          Show this help message
  --baseline      Create/update baseline constraint file (db_constraints.txt)
  --strict        Exit with error if any unknown constraints found
  --quiet         Reduce output verbosity

Environment:
  DATABASE_URL    Required: Postgres connection string

Examples:
  ./scripts/audit-db-constraints.sh
  ./scripts/audit-db-constraints.sh --baseline
  ./scripts/audit-db-constraints.sh --strict
EOF
            exit 0
            ;;
        --baseline)
            BASELINE=true
            shift
            ;;
        --strict)
            STRICT=true
            shift
            ;;
        --quiet)
            QUIET=true
            shift
            ;;
        *)
            echo "Unknown option: $1" >&2
            echo "Use --help for usage information" >&2
            exit 1
            ;;
    esac
done

# Logging functions
log_info() {
    if [[ "$QUIET" != true ]]; then
        echo -e "${BLUE}ℹ${NC} $1"
    fi
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

# Check if DATABASE_URL is set
if [[ -z "${DATABASE_URL:-}" ]]; then
    log_error "DATABASE_URL environment variable is not set"
    echo "Please set DATABASE_URL to your Postgres connection string" >&2
    exit 1
fi

# Check if migrations directory exists
if [[ ! -d "$MIGRATIONS_DIR" ]]; then
    log_error "Migrations directory not found: $MIGRATIONS_DIR"
    exit 1
fi

log_info "Starting database constraint audit..."
log_info "Database: $(echo "$DATABASE_URL" | sed 's|://.*@|://***:***@|')"
log_info "Migrations directory: $MIGRATIONS_DIR"

# Create temporary directory
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Step 1: Dump all DB constraints to file
log_info "Dumping database constraints..."
cat > "$TEMP_DIR/dump_constraints.sql" << 'SQL'
SELECT n.nspname || '.' || rel.relname         AS table,
       c.conname                               AS constraint,
       c.contype                               AS type,  -- p=PK, f=FK, u=UNIQUE, c=CHECK
       pg_get_constraintdef(c.oid)             AS definition
FROM pg_constraint c
JOIN pg_class rel       ON rel.oid = c.conrelid
JOIN pg_namespace n     ON n.oid = rel.relnamespace
WHERE n.nspname NOT IN ('pg_catalog','information_schema')
ORDER BY 1,2;
SQL

if ! psql "$DATABASE_URL" -At -f "$TEMP_DIR/dump_constraints.sql" > "$TEMP_DIR/db_constraints.txt" 2>/dev/null; then
    log_error "Failed to connect to database or execute constraint dump query"
    echo "Please check your DATABASE_URL and database connectivity" >&2
    exit 1
fi

log_success "Found $(wc -l < "$TEMP_DIR/db_constraints.txt") constraints in database"

# Step 2: Extract constraint names from Drizzle migrations
log_info "Extracting constraint names from Drizzle migrations..."
if ! find "$MIGRATIONS_DIR" -name "*.sql" -exec grep -RhoiE 'constraint[[:space:]]+([a-zA-Z0-9_"]+)' {} \; 2>/dev/null | \
     sed -E 's/.*constraint[[:space:]]+"?([a-zA-Z0-9_]+)"?/\1/' | \
     sort -u > "$TEMP_DIR/drizzle_constraints.txt"; then
    log_error "Failed to extract constraints from migrations"
    exit 1
fi

log_success "Found $(wc -l < "$TEMP_DIR/drizzle_constraints.txt") constraints in migrations"

# Step 3: Compare DB vs Drizzle constraints
log_info "Comparing database vs migration constraints..."

# Extract just the constraint names from DB dump (column 2)
cut -d'|' -f2 "$TEMP_DIR/db_constraints.txt" | sort -u > "$TEMP_DIR/db_constraint_names.txt"

# Find constraints in DB but not in migrations
comm -23 "$TEMP_DIR/db_constraint_names.txt" "$TEMP_DIR/drizzle_constraints.txt" > "$TEMP_DIR/unknown_constraints.txt"

UNKNOWN_COUNT=$(wc -l < "$TEMP_DIR/unknown_constraints.txt")

# Handle baseline creation
if [[ "$BASELINE" == true ]]; then
    cp "$TEMP_DIR/db_constraints.txt" "db_constraints.txt"
    log_success "Created baseline file: db_constraints.txt"
    if [[ "$UNKNOWN_COUNT" -gt 0 ]]; then
        log_warning "Found $UNKNOWN_COUNT unknown constraints - they are now part of the baseline"
    fi
    exit 0
fi

# Report results
if [[ "$UNKNOWN_COUNT" -eq 0 ]]; then
    log_success "Audit passed: No unknown constraints found!"
    log_success "All database constraints are managed by Drizzle migrations"
else
    log_warning "Found $UNKNOWN_COUNT constraint(s) in database but not in migrations:"
    echo
    echo "Unknown constraints:"
    cat "$TEMP_DIR/unknown_constraints.txt" | sed 's/^/  - /'
    echo

    if [[ "$STRICT" == true ]]; then
        log_error "Strict mode enabled - exiting with error due to unknown constraints"
        exit 1
    else
        log_warning "Consider adding these constraints to your Drizzle migrations"
        log_info "Or run with --baseline to accept current state as baseline"
    fi
fi

# Show summary if not quiet
if [[ "$QUIET" != true ]]; then
    echo
    log_info "Summary:"
    echo "  Database constraints: $(wc -l < "$TEMP_DIR/db_constraint_names.txt")"
    echo "  Migration constraints: $(wc -l < "$TEMP_DIR/drizzle_constraints.txt")"
    echo "  Unknown constraints: $UNKNOWN_COUNT"
    echo
    log_info "Files created in current directory:"
    echo "  - db_constraints.txt (database dump)"
    echo "  - drizzle_constraints.txt (migration constraints)"
    if [[ "$UNKNOWN_COUNT" -gt 0 ]]; then
        echo "  - unknown_constraints.txt (constraints not in migrations)"
    fi
fi

log_success "Audit completed successfully"
