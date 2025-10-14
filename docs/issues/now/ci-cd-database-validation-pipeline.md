# TODO: CI/CD Database Validation Pipeline

## Goal

Integrate database validation scripts into CI/CD pipeline to catch schema mismatches before deployment.

## Problem We Solved

We recently encountered a critical issue where:

- **Database**: `storage_edges.memory_id` was an `integer` (old schema)
- **Code**: Expected `storage_edges.memory_id` to be a `uuid` (new schema)

This caused runtime errors like:

```
invalid input syntax for type integer: "2fdc73f8-c0d4-4273-81db-847664170c8d"
```

## The Solution

These scripts help catch schema mismatches **before** they cause runtime errors:

1. **`validate-schema.ts`** - Catches type mismatches, missing columns, etc.
2. **`verify-constraints.ts`** - Ensures constraints defined in schema exist in database

## Scripts to Integrate

- `db/verify-constraints.ts` - Database constraint validation
- `db/migrate.ts` - Database migrations
- `db/test-migration.ts` - Migration testing

## Proposed CI/CD Steps

```yaml
# Example GitHub Actions workflow
- name: Validate Database Schema
  run: |
    npm run db:validate
    npm run db:verify
    npm run db:migrate:test
```

## What Each Script Checks

### `validate-schema.ts`

- ✅ Table existence
- ✅ Column data types (e.g., `uuid` vs `integer`)
- ✅ Column nullability
- ✅ Primary key constraints
- ✅ Default values

### `verify-constraints.ts`

- ✅ CHECK constraints (e.g., `memory_assets_bytes_positive`)
- ✅ Custom constraints defined in schema
- ⚠️ Note: Drizzle cannot automatically verify these constraints

## Exit Codes

- `0` - All checks passed
- `1` - Issues detected (mismatches, missing constraints, etc.)

## Benefits

- Catch schema mismatches before deployment
- Prevent runtime errors from type mismatches
- Ensure database constraints are properly applied
- Validate migrations work correctly

## Usage in Development Workflow

### After Schema Changes

```bash
# 1. Generate migration
npm run db:generate

# 2. Validate schema matches
npm run db:validate

# 3. Push changes
npm run db:push

# 4. Verify constraints
npm run db:verify
```

### Before Deployment

```bash
# Run both validation scripts
npm run db:validate && npm run db:verify
```

## Troubleshooting

### Schema Mismatch Detected

1. Check what changes are needed: `npx drizzle-kit push --dry-run`
2. Force push changes: `npx drizzle-kit push --force`
3. Or create proper migration: `npx drizzle-kit generate`

### Missing Constraints

1. Check if constraints are defined in `src/db/schema.ts`
2. If defined, run: `npx drizzle-kit push --force`
3. If not defined, add them to the schema

### Data Loss Warnings

- These scripts will warn about potential data loss
- In development, you can usually accept data loss
- In production, create proper migrations with data transformation

## Status

**Pending implementation** - Ready for CI/CD integration
