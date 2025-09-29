# Logger Circular Reference Crash Issue

## 🚨 CRITICAL BUG: API Endpoints Crashing Due to Logger Serialization

**Status**: 🔴 ACTIVE  
**Priority**: HIGH  
**Impact**: Complete API failure for memories endpoint  
**Date**: 2025-09-28

## Problem Description

The `/api/memories` endpoint is returning 500 errors due to logger serialization failures. The error occurs when the logger tries to serialize complex database objects that contain circular references.

### Error Details

```
TypeError: Converting circular structure to JSON
    --> starting at object with constructor 'PgTable'
    |     property 'id' -> object with constructor 'PgUUID'
    --- property 'table' closes the circle
    at JSON.stringify (<anonymous>)
    at Logger.formatMessage (/path/to/logger.ts:1495:47)
    at Logger.info (/path/to/logger.ts:1505:31)
    at handleApiMemoryGet (/path/to/memories/get.ts:2745:138)
```

## Root Cause Analysis

### What Happened

1. **Logger Migration**: When replacing `console.log` with `logger.info`, complex objects were passed directly as the second parameter
2. **Parameter Mismatch**: Logger expects `(message, context?)` but received `(message, complexObject)`
3. **Serialization Failure**: `JSON.stringify()` in `formatMessage()` fails on Drizzle ORM objects with circular references
4. **API Crash**: The serialization error crashes the entire API endpoint

### Technical Details

- **Drizzle ORM Objects**: Database query results contain circular references between table definitions and column types
- **Logger Implementation**: Uses `JSON.stringify(context)` to format log messages
- **Circular Reference**: `PgTable.id` → `PgUUID` → `PgTable` creates infinite loop

## Affected Code

### Problematic Logger Calls

```typescript
// ❌ WRONG - passes complex object directly
logger.info("🔍 API: Sample memory:", userMemories[0]);
logger.info("🔍 API: Sample returned memory:", memoriesWithShareInfo[0]);
logger.info("📨 Share request body:", body);
logger.info("🔍 Request body:", parsedBody);
```

### Correct Pattern

```typescript
// ✅ CORRECT - wraps object in context
logger.info("🔍 API: Sample memory:", undefined, { sampleMemory: userMemories[0] });
logger.info("📨 Share request body:", undefined, { body });
```

## Impact Assessment

### Immediate Impact

- **API Endpoint**: `/api/memories` returns 500 errors
- **User Experience**: Dashboard cannot load memories
- **Frontend**: Shows "Failed to list memories" error
- **Application**: Core functionality broken

### Affected Files

- `src/app/api/memories/get.ts` - Main memories API
- `src/app/api/memories/[id]/share/route.ts` - Share functionality
- `src/test/simple-endpoint.test.ts` - Test endpoints

## Solution Strategy

### Immediate Fix (Temporary)

1. **Comment Out Problematic Loggers**: Remove or comment out logger calls that pass complex objects
2. **Restore API Functionality**: Get the endpoint working again
3. **Test Endpoint**: Verify memories API returns data correctly

### Long-term Fix (Permanent)

1. **Logger Parameter Validation**: Add runtime checks for circular references
2. **Safe Serialization**: Implement `JSON.stringify` with circular reference handling
3. **Object Sanitization**: Create utility to safely serialize database objects
4. **Logger Enhancement**: Add `safeStringify` method to logger class

## Implementation Plan

### Phase 1: Emergency Fix

- [ ] Comment out problematic logger calls in memories API
- [ ] Test API endpoint functionality
- [ ] Verify dashboard loads correctly

### Phase 2: Logger Enhancement

- [ ] Add circular reference detection to logger
- [ ] Implement safe serialization utility
- [ ] Update logger to handle complex objects gracefully

### Phase 3: Code Review

- [ ] Audit all logger calls for similar issues
- [ ] Create linting rules to prevent future issues
- [ ] Add unit tests for logger serialization

## Prevention Measures

### Code Guidelines

1. **Never pass complex objects directly to logger**
2. **Always wrap objects in context parameter**
3. **Use simple, serializable data in logs**
4. **Test logger calls with actual data**

### Linting Rules

```typescript
// Add ESLint rule to catch problematic patterns
"no-direct-object-logging": "error"
```

### Logger Best Practices

```typescript
// ✅ GOOD
logger.info("User created", undefined, { userId: user.id, email: user.email });

// ❌ BAD
logger.info("User created", user); // Complex object with circular refs
```

## Testing Strategy

### Unit Tests

- Test logger with circular reference objects
- Test logger with Drizzle ORM objects
- Test logger serialization edge cases

### Integration Tests

- Test memories API with actual database objects
- Test all API endpoints for logger issues
- Test error handling in logger

## Related Issues

- **Logger System Implementation**: Initial logger creation
- **TypeScript Error Resolution**: Previous logger parameter fixes
- **API Error Handling**: General error handling improvements

## Timeline

- **Immediate**: Fix API crash (1 hour)
- **Short-term**: Logger enhancement (1 day)
- **Long-term**: Prevention measures (1 week)

## Notes

This issue highlights the importance of:

1. **Careful object serialization** in logging systems
2. **Testing with real data** during development
3. **Defensive programming** for complex object handling
4. **Proper error handling** in utility functions

The logger system itself is sound, but needs better handling of complex objects with circular references.
