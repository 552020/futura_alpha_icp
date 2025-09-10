# Database Constraint Audit Script

## Overview

The `audit-db-constraints.sh` script performs a **3-minute database schema drift audit** to catch constraints that exist in your Postgres database but weren't created by Drizzle migrations.

This ensures **Drizzle remains the single source of truth** for your database schema and helps prevent surprise schema drift.

## Quick Start

```bash
# Basic audit (requires DATABASE_URL)
./scripts/audit-db-constraints.sh

# Create baseline file for CI
./scripts/audit-db-constraints.sh --baseline

# Strict mode (fail on unknown constraints)
./scripts/audit-db-constraints.sh --strict

# Quiet mode for CI
./scripts/audit-db-constraints.sh --quiet
```

## What It Does

1. **Dumps all constraints** from your Postgres database
2. **Extracts constraint names** from your Drizzle migration files
3. **Compares them** to find "out-of-band" constraints
4. **Reports discrepancies** with actionable recommendations

## Output Files

The script creates these files in the current directory:

- `db_constraints.txt` - All constraints found in the database
- `drizzle_constraints.txt` - Constraint names from migrations
- `unknown_constraints.txt` - Constraints in DB but not migrations (if any)

## CI Integration

Add this to your CI pipeline:

```yaml
# .github/workflows/ci.yml
- name: Audit database constraints
  run: ./scripts/audit-db-constraints.sh --strict
  env:
    DATABASE_URL: ${{ secrets.DATABASE_URL }}
```

## Common Scenarios

### ✅ Clean Database

```
✓ Audit passed: No unknown constraints found!
✓ All database constraints are managed by Drizzle migrations
```

### ⚠️ Schema Drift Detected

```
⚠ Found 2 constraint(s) in database but not in migrations:

Unknown constraints:
  - user_email_unique_idx
  - gallery_created_at_check

⚠ Consider adding these constraints to your Drizzle migrations
ℹ Or run with --baseline to accept current state as baseline
```

## Environment Variables

- `DATABASE_URL` - Required: Postgres connection string

## Exit Codes

- `0` - Success (no unknown constraints or baseline created)
- `1` - Error (missing DATABASE_URL, DB connection failed, or strict mode violations)

## Troubleshooting

### "Migrations directory not found"

Make sure you're running from the repository root and the path `src/nextjs/src/db/migrations` exists.

### "Failed to connect to database"

Check your `DATABASE_URL` and database connectivity.

### Too many unknown constraints?

Run with `--baseline` to accept current state, then investigate which constraints should be in migrations vs removed.

## MVP Benefits

- **Prevents schema drift** before it causes production issues
- **Fast to run** (~3 seconds for typical databases)
- **CI/CD ready** with proper exit codes
- **Actionable output** with specific constraint names
- **Zero dependencies** (uses only psql and standard Unix tools)


