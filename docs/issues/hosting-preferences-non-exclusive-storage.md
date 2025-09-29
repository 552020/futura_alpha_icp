# Hosting Preferences: Non-Exclusive Storage Options

## Problem

Currently, the `userHostingPreferences` table stores a single `blobHosting` value per user, which means users can only choose ONE storage provider (e.g., 's3', 'icp', 'vercel_blob'). This is too restrictive for users who want:

1. **Fallback storage** - Primary S3 with ICP as backup
2. **Redundancy** - Store in multiple places simultaneously
3. **Flexibility** - Different storage for different use cases

## Current Schema

```typescript
export const userHostingPreferences = pgTable("user_hosting_preferences", {
  // ... other fields ...
  blobHosting: blob_hosting_t("blob_hosting").default("s3").notNull(), // SINGLE VALUE
  // ... other fields ...
});
```

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

## Decision: JSONB Array

**Chosen because:**

1. **Schema Symmetry** - Maintains the beautiful symmetry of the current design
2. **Type Safety** - Drizzle provides full type safety with `$type<BlobHosting[]>()`
3. **Simple Migration** - Just change one field type
4. **PostgreSQL-Only** - We're committed to PostgreSQL, so the limitation is acceptable
5. **Performance** - PostgreSQL JSONB is optimized and fast

## Implementation Notes

- Added comprehensive documentation in schema comments
- Created `BlobHosting` TypeScript type for type safety
- Default value is `['s3']` to maintain backward compatibility
- Future: Could extend to support database hosting arrays as well

## Future Considerations

1. **Database Hosting Arrays** - Similar approach for database redundancy
2. **Storage Metadata** - Add fields like `enabled`, `cost`, `performance` to each storage option
3. **Migration Strategy** - Convert existing single values to arrays during migration
4. **API Changes** - Update API endpoints to handle arrays instead of single values

## Related Files

- `src/db/schema.ts` - Schema definition
- `src/hooks/use-storage-preferences.ts` - React Query hooks
- `src/app/api/me/hosting-preferences/route.ts` - API endpoints
- `src/services/upload/single-file-processor.ts` - Upload logic
- `src/services/upload/multiple-files-processor.ts` - Multiple file upload logic



