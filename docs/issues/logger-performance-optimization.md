# Logger Performance Optimization Issue

## Problem

The current logger implementation has a performance issue where context objects and expensive operations are executed even when logging is disabled.

### Current Behavior

```typescript
logger.info("📤 Upload routing decision", {
  service: "upload",
  selectedProvider: userBlobHostingPreferences[0], // ❌ Always executed
  fileName: file.name, // ❌ Always executed
  fileSize: file.size, // ❌ Always executed
  fileType: file.type, // ❌ Always executed
  isOnboarding, // ❌ Always executed
  mode, // ❌ Always executed
});
```

**What happens:**

1. `logger.info()` is called
2. Context object `{ service: 'upload', ... }` is **created immediately**
3. Property access (`file.name`, `file.size`, etc.) happens **immediately**
4. Array access (`userBlobHostingPreferences[0]`) happens **immediately**
5. Only then `shouldLog(LogLevel.INFO)` is checked
6. If logging is disabled, the object is discarded

### Performance Impact

- **Object allocation**: Context objects created even when disabled
- **Property access**: File properties accessed even when disabled
- **Array operations**: Array indexing happens even when disabled
- **Function calls**: Any function calls in context creation happen even when disabled

## Proposed Solutions

### Option 1: Lazy Evaluation (Recommended)

```typescript
// New signature
info(message: string, contextFactory?: () => any): void {
  if (this.shouldLog(LogLevel.INFO)) {
    const context = contextFactory ? contextFactory() : undefined;
    console.info(this.formatPrefix('INFO'), message, context);
  }
}

// Usage
logger.info('📤 Upload routing decision', () => ({
  service: 'upload',
  selectedProvider: userBlobHostingPreferences[0],  // ✅ Only executed if logging enabled
  fileName: file.name,                              // ✅ Only executed if logging enabled
  fileSize: file.size,                              // ✅ Only executed if logging enabled
  fileType: file.type,                              // ✅ Only executed if logging enabled
  isOnboarding,                                     // ✅ Only executed if logging enabled
  mode,                                             // ✅ Only executed if logging enabled
}));
```

### Option 2: Conditional Logging

```typescript
// In calling code
if (logger.shouldLog(LogLevel.INFO)) {
  logger.info("📤 Upload routing decision", {
    service: "upload",
    selectedProvider: userBlobHostingPreferences[0],
    // ... only created if logging is enabled
  });
}
```

### Option 3: Keep Current (Acceptable for Most Cases)

The performance impact is usually minimal unless logging in tight loops or with very expensive object creation.

## Implementation Plan

1. **Update logger methods** to support lazy evaluation
2. **Update all logger calls** to use context factories where expensive operations are involved
3. **Add performance tests** to measure the improvement
4. **Document the new pattern** for future logger usage

## Files to Update

- `src/nextjs/src/lib/logger.ts` - Core logger implementation
- `src/nextjs/src/hooks/use-hosting-preferences.ts` - Preference logging
- `src/nextjs/src/services/upload/single-file-processor.ts` - Upload routing logs
- `src/nextjs/src/services/upload/multiple-files-processor.ts` - Multiple file routing logs
- `src/nextjs/src/app/api/me/hosting-preferences/route.ts` - API preference logs

## Service Flags Available

### Core Service Flags

- `ENABLE_UI_LOGGING` - UI components and interactions (master UI switch)
- `ENABLE_BACKEND_LOGGING` - Backend API and processing (master backend switch)

### Backend Services (require ENABLE_BACKEND_LOGGING = true)

- `ENABLE_UPLOAD_LOGGING` - Upload routing and processing logs
- `ENABLE_DATABASE_LOGGING` - Database operation logs
- `ENABLE_AUTH_LOGGING` - Authentication and user management logs
- `ENABLE_ASSET_LOGGING` - Asset processing and thumbnail generation logs
- `ENABLE_S3_LOGGING` - S3 presigned URL generation and storage logs

### UI Services (require ENABLE_UI_LOGGING = true)

- `ENABLE_DASHBOARD_LOGGING` - Dashboard state and API calls
- `ENABLE_MEMORY_PROCESSING_LOGGING` - Memory processing and folder grouping
- `ENABLE_RENDERING_LOGGING` - Component rendering logs

### Cross-cutting Concerns (require multiple flags)

- `ENABLE_HOSTING_PREFERENCES` - Hosting preference changes and routing (requires UI + HOSTING_PREFERENCES)

## Priority

**Medium** - Performance optimization that improves efficiency but doesn't affect functionality.

## Status

**Open** - TODO comment added to logger.ts, issue documented.
