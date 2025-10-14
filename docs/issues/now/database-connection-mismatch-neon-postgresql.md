# Database Connection Mismatch - Neon Driver with PostgreSQL Connection String

**Priority:** High  
**Type:** Bug  
**Component:** Database/Authentication  
**Created:** 2025-01-14  
**Status:** Open

## Problem Description

Google OAuth authentication is failing due to a database connection mismatch. The application is configured to use Neon database driver but is attempting to connect to a local PostgreSQL database.

## Root Cause

**File:** `src/nextjs/src/db/db.ts`  
**Line 20:** `const sql = neon(connectionString!);`

The `neon()` function expects Neon cloud database URLs but is receiving a local PostgreSQL connection string from `.env.local`:

```
DATABASE_URL=postgresql://futura_user:futura_password@localhost:5432/futura_dev
```

## Error Details

```
[auth][error] AdapterError: Failed query: select "account"."userId"...
[auth][cause]: Error: Failed query: select "account"."userId", "account"."type"...
```

## Impact

- Google OAuth authentication completely broken
- Users cannot sign in
- Authentication flow fails at database lookup step

## Proposed Solutions

### Option 1: Switch to Neon Cloud Database

- Uncomment Neon URLs in `.env.local`
- Keep current `db.ts` configuration
- **Pros:** No code changes needed
- **Cons:** Requires cloud database access

### Option 2: Switch to PostgreSQL Driver for Local Development

- Change `db.ts` to use `drizzle-orm/postgres-js`
- Install `postgres` package
- Keep local Docker database
- **Pros:** Local development independence
- **Cons:** Requires code changes

## Files Affected

- `src/nextjs/src/db/db.ts` (lines 1, 2, 20)
- `src/nextjs/.env.local` (database URLs)

## Acceptance Criteria

- [ ] Google OAuth authentication works without errors
- [ ] Database connection is consistent (either all Neon or all PostgreSQL)
- [ ] Local development environment is functional

## Technical Details

### Current Configuration

```typescript
// db.ts - Using Neon driver
import { drizzle } from "drizzle-orm/neon-http";
import { neon } from "@neondatabase/serverless";
const sql = neon(connectionString!);
```

### Environment Variables

```bash
# .env.local - Using PostgreSQL connection string
DATABASE_URL=postgresql://futura_user:futura_password@localhost:5432/futura_dev
```

### Error Stack Trace

```
[auth][error] AdapterError: Read more at https://errors.authjs.dev#adaptererror
[auth][cause]: Error: Failed query: select "account"."userId", "account"."type", "account"."provider", "account"."providerAccountId"...
```

## Next Steps

1. **Immediate:** Choose between Option 1 or Option 2
2. **Implement:** Make the necessary configuration changes
3. **Test:** Verify Google OAuth authentication works
4. **Document:** Update development setup documentation
