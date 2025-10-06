# Hosting Preferences: Non-Exclusive Storage Options

## ✅ RESOLVED - IMPLEMENTED

**Status**: Completed and deployed  
**Implementation Date**: December 2024  
**Solution**: JSONB Array approach with full migration

## Problem (RESOLVED)

~~Currently, the `userHostingPreferences` table stores a single `blobHosting` value per user, which means users can only choose ONE storage provider (e.g., 's3', 'icp', 'vercel_blob'). This is too restrictive for users who want:~~

1. **Fallback storage** - Primary S3 with ICP as backup ✅ **IMPLEMENTED**
2. **Redundancy** - Store in multiple places simultaneously ✅ **IMPLEMENTED**  
3. **Flexibility** - Different storage for different use cases ✅ **IMPLEMENTED**

## ✅ IMPLEMENTED Schema

```typescript
export const userHostingPreferences = pgTable("user_hosting_preferences", {
  // ... other fields ...
  databaseHosting: jsonb('database_hosting').$type<DatabaseHosting[]>().default(['neon']).notNull(),
  blobHosting: jsonb('blob_hosting').$type<BlobHosting[]>().default(['s3']).notNull(),
  // ... other fields ...
});
```

**Key Changes:**
- ✅ `blobHosting` now supports arrays: `BlobHosting[]`
- ✅ `databaseHosting` also supports arrays: `DatabaseHosting[]`  
- ✅ Type-safe with Drizzle `$type<BlobHosting[]>()`
- ✅ Backward compatible with migration from single values to arrays

## Solutions Discussed

### Option 1: JSONB Array (IMPLEMENTED ✅)

**Schema Change:**

```typescript
blobHosting: jsonb('blob_hosting').$type<BlobHosting[]>().default(['s3']).notNull(),
```

**Pros:**

- ✅ Maintains schema symmetry (all hosting types in one table)
- ✅ Type-safe with Drizzle `$type<BlobHosting[]>()`
- ✅ Simple migration from current schema
- ✅ Easy to query with PostgreSQL JSONB operators
- ✅ Supports priority order: `['s3', 'icp', 'vercel_blob']`

**Cons:**

- ❌ PostgreSQL-specific (not standard SQL)
- ❌ Won't work with MySQL, SQLite, etc.
- ❌ JSON parsing overhead (minimal)

**Data Examples:**

```json
// Single storage
{ "blobHosting": ["s3"] }

// Multiple with fallback
{ "blobHosting": ["s3", "icp", "vercel_blob"] }

// ICP-first with S3 fallback
{ "blobHosting": ["icp", "s3"] }
```

### Option 2: Separate Table (Normalized)

**Schema:**

```typescript
export const userStoragePreferences = pgTable("user_storage_preferences", {
  id: uuid("id").primaryKey().defaultRandom(),
  userId: text("user_id")
    .notNull()
    .references(() => users.id, { onDelete: "cascade" }),
  storageType: blob_hosting_t("storage_type").notNull(),
  priority: integer("priority").notNull(), // 1 = first choice, 2 = fallback
  enabled: boolean("enabled").default(true).notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});
```

**Pros:**

- ✅ Standard SQL (works with any database)
- ✅ Fully normalized design
- ✅ Indexed queries by userId and priority
- ✅ No JSON parsing overhead
- ✅ Easy to add metadata (enabled/disabled, priority)

**Cons:**

- ❌ Breaks schema symmetry
- ❌ Requires joins for simple queries
- ❌ More complex to query
- ❌ Requires migration to separate table

**Data Example:**

```
userId | storageType | priority | enabled
user1  | s3          | 1        | true
user1  | icp         | 2        | true
user1  | vercel_blob | 3        | false
```

### Option 3: Comma-Separated String

**Schema:**

```typescript
blobHosting: text('blob_hosting').default('s3').notNull(),
```

**Pros:**

- ✅ Standard SQL
- ✅ Simple implementation
- ✅ Maintains schema symmetry

**Cons:**

- ❌ No type safety
- ❌ Manual parsing required
- ❌ No validation of values
- ❌ Harder to query

**Data Examples:**

```
"s3"
"s3,icp"
"icp,s3,vercel_blob"
```

### Option 4: Union of Enums

**Schema:**

```typescript
// Create union enum
export const storage_hosting_t = pgEnum('storage_hosting_t', [
  's3', 'vercel_blob', 'icp', 'arweave', 'ipfs', 'neon'
]);

blobHosting: storage_hosting_t('blob_hosting').notNull(),
```

**Pros:**

- ✅ Type-safe at database level
- ✅ Standard SQL enums

**Cons:**

- ❌ Duplicates values from original enums
- ❌ No way to enforce valid combinations
- ❌ Maintenance overhead
- ❌ Still single value only

## ✅ IMPLEMENTED: JSONB Array

**Successfully implemented because:**

1. ✅ **Schema Symmetry** - Maintains the beautiful symmetry of the current design
2. ✅ **Type Safety** - Drizzle provides full type safety with `$type<BlobHosting[]>()`
3. ✅ **Simple Migration** - Successfully migrated existing single values to arrays
4. ✅ **PostgreSQL-Only** - We're committed to PostgreSQL, so the limitation is acceptable
5. ✅ **Performance** - PostgreSQL JSONB is optimized and fast

## ✅ Implementation Completed

- ✅ Added comprehensive documentation in schema comments
- ✅ Created `BlobHosting` and `DatabaseHosting` TypeScript types for type safety
- ✅ Default values: `blobHosting: ['s3']`, `databaseHosting: ['neon']` for backward compatibility
- ✅ **Migration completed**: `0003_needy_fantastic_four.sql` converts existing single values to arrays
- ✅ **API updated**: `/api/me/hosting-preferences` handles arrays
- ✅ **Frontend updated**: React hooks support array preferences
- ✅ **Type safety**: Full TypeScript support with proper array types

## ✅ Implementation Details

### Migration Strategy (COMPLETED)
- ✅ **Migration file**: `0003_needy_fantastic_four.sql`
- ✅ **Strategy**: Convert existing single values to arrays using `jsonb_build_array()`
- ✅ **Backward compatibility**: Existing data preserved and converted
- ✅ **Zero downtime**: Migration handles existing data seamlessly

### API Changes (COMPLETED)
- ✅ **API endpoint**: `/api/me/hosting-preferences` updated to handle arrays
- ✅ **Type safety**: Full TypeScript support for array preferences
- ✅ **Default handling**: Proper defaults for new users

### Frontend Integration (COMPLETED)
- ✅ **React hooks**: `use-hosting-preferences.ts` supports arrays
- ✅ **Type definitions**: `BlobHosting[]` and `DatabaseHosting[]` types
- ✅ **UI components**: Settings page supports multiple storage options

## Future Considerations

1. ✅ **Database Hosting Arrays** - **COMPLETED** - Both database and blob hosting now support arrays
2. **Storage Metadata** - Add fields like `enabled`, `cost`, `performance` to each storage option
3. **Priority Management** - UI for reordering storage preferences by priority
4. **Fallback Logic** - Implement automatic fallback when primary storage fails

## ✅ Related Files (IMPLEMENTED)

- ✅ `src/nextjs/src/db/schema.ts` - Schema definition with JSONB arrays
- ✅ `src/nextjs/src/hooks/use-hosting-preferences.ts` - React Query hooks for arrays
- ✅ `src/nextjs/src/app/api/me/hosting-preferences/route.ts` - API endpoints handling arrays
- ✅ `src/nextjs/src/db/migrations/0003_needy_fantastic_four.sql` - Migration script
- ✅ `src/nextjs/src/app/[lang]/user/settings/page.tsx` - UI for managing preferences



