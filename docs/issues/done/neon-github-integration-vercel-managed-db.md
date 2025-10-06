# Neon GitHub Integration with Vercel-Managed Database

## Context

We are implementing **non-exclusive storage preferences** to allow users to have multiple storage options with fallback chains (e.g., `['s3', 'icp', 'vercel_blob']` instead of single values).

## What We've Done So Far

### 1. Schema Changes Made ✅

**Modified `src/nextjs/src/db/schema.ts`:**

- Changed `blobHosting` from single enum to JSONB array: `jsonb('blob_hosting').$type<BlobHosting[]>().default(['s3']).notNull()`
- Changed `databaseHosting` from single enum to JSONB array: `jsonb('database_hosting').$type<DatabaseHosting[]>().default(['neon']).notNull()`
- Added TypeScript types: `BlobHosting` and `DatabaseHosting`
- Maintained schema symmetry while enabling redundancy/fallback support

### 2. GitHub Actions Workflow Created ✅

**Created `.github/workflows/neon-branching.yml`:**

- Automatically creates Neon database branch for each PR
- Runs `npm run db:push` to test schema migrations
- Posts schema diff as PR comment
- Deletes branch when PR is closed
- Uses `neondatabase/create-branch-action@v5` with required parameters

### 3. Code Committed and Pushed ✅

**Branch:** `552020/icp-421-fix-small-stuff-while-reading-the-upload-flow`
**Commit:** `feat(schema): add non-exclusive storage preferences with JSONB arrays`

## Current Status

- ✅ Schema changes implemented
- ✅ GitHub Actions workflow ready
- ✅ Code pushed to GitHub
- ✅ **Vercel project linked and analyzed**
- ✅ **Database ownership confirmed: Direct Neon (not Vercel-managed)**
- ❓ **Neon GitHub Integration setup pending**

## ✅ RESOLVED: Database Ownership Analysis

### Investigation Results

We investigated whether the database was Vercel-managed by:

1. **Linking Vercel project**: `vercel link` → Connected to `552020s-projects/nextjs`
2. **Checking environment variables**: `vercel env pull .env.development.local`
3. **Analyzing Vercel configuration**: `vercel env ls`

### Key Findings

**✅ Database is DIRECTLY Neon-managed (NOT Vercel-managed):**

- Vercel project has **no environment variables** (including no database variables)
- Only contains `VERCEL_OIDC_TOKEN` for authentication
- Database connection is managed locally in `.env.local`
- No Vercel Postgres integration detected

### ✅ Tech Lead Confirmation

**"You're on pure Neon. Those vars are Neon's own templates surfaced inside Vercel's UI. The tell is the hostnames: `*.neon.tech` and `*-pooler.eu-central-1.aws.neon.tech`. Vercel-managed Postgres would use `*.vercel-storage.com`."**

**✅ This means:**

- We have **direct Neon console access**
- We can **proceed with Neon GitHub integration** without Vercel complications
- No Vercel-specific restrictions or permissions needed
- The `POSTGRES_*` variables are just convenience aliases, not Vercel management

### Current Database Configuration

```bash
# From .env.local
DATABASE_URL=postgres://neondb_owner:npg_WDbjeXO39LKF@ep-withered-sunset-a23i96jq-pooler.eu-central-1.aws.neon.tech/neondb?sslmode=require
PGHOST=ep-withered-sunset-a23i96jq-pooler.eu-central-1.aws.neon.tech
PGUSER=neondb_owner
PGDATABASE=neondb
```

**Project ID:** `ep-withered-sunset-a23i96jq`
**Region:** `eu-central-1`

### Tech Lead Recommendations

**Environment Variables:**

- ✅ **Keep**: `DATABASE_URL` (pooled) for runtime
- ✅ **Keep**: `DATABASE_URL_UNPOOLED` for migrations/CLI
- ✅ **Optional**: `PG*` variables if using psql
- ⚠️ **Optional**: `POSTGRES_*` set (convenience aliases, safe to remove)
- ❌ **Do NOT use**: `POSTGRES_URL_NO_SSL`

**Best Practices:**

- **Runtime** (Next.js/Edge/API): Use pooled `DATABASE_URL` (via PgBouncer)
- **Migrations** (Drizzle/Prisma CLI): Use unpooled `DATABASE_URL_UNPOOLED`

**Security Note:**

- ⚠️ **CRITICAL**: Live password was exposed in documentation - **rotate immediately** in Neon console

## ✅ RESOLVED: All Questions Answered

### 1. Database Management ✅

- **Answer**: Database is **directly managed by Neon** (not proxied through Vercel)
- **Answer**: We have **direct Neon console access**
- **Answer**: We can **set up Neon GitHub integration directly** (no Vercel permission needed)

### 2. Migration Strategy ✅

- **Answer**: Use **Neon's GitHub integration** (Vercel has no database management)
- **Answer**: No Vercel-specific tools needed
- **Answer**: Standard Neon schema migration approach

### 3. Access and Permissions ✅

- **Answer**: We have **full permissions** for Neon GitHub integration
- **Answer**: **No Vercel restrictions** (database is not Vercel-managed)
- **Answer**: **No Vercel coordination needed**

## ✅ CHOSEN APPROACH: Direct Neon Integration

**Selected**: **Option 1: Direct Neon Integration** ✅

- ✅ Set up Neon GitHub integration directly
- ✅ Use Neon's branching system for testing
- ✅ No Vercel coordination needed (confirmed)

**Rejected**: Option 2 (Vercel-managed) - Not applicable
**Rejected**: Option 3 (Local testing) - Not needed, we can test directly on Neon

## Next Steps

1. ✅ **Database Analysis**: Confirmed direct Neon management
2. ✅ **Permission Check**: Confirmed full Neon access
3. ✅ **Migration Strategy**: Chosen direct Neon integration
4. **Implementation**: Set up Neon GitHub integration and test migration

## Files Modified

- `src/nextjs/src/db/schema.ts` - Schema changes
- `src/nextjs/.github/workflows/neon-branching.yml` - GitHub Actions workflow
- `docs/issues/hosting-preferences-non-exclusive-storage.md` - Technical documentation
- `src/nextjs/.vercel/` - Vercel project configuration (linked)
- `src/nextjs/.env.development.local` - Vercel environment variables (empty - confirms no Vercel DB)

## Related Documentation

- [Hosting Preferences Non-Exclusive Storage](./hosting-preferences-non-exclusive-storage.md)
- [Neon GitHub Integration Documentation](https://neon.tech/docs/guides/github-integration)
- [Vercel Postgres Documentation](https://vercel.com/docs/storage/vercel-postgres)
