# Logger System Architecture

## Overview

The logger system provides structured, configurable logging with hierarchical control over what gets logged. It uses a two-tier architecture with master flags, context flags, and service-specific flags to provide fine-grained control over logging output.

## Architecture Components

### 1. Logger Types

#### SimpleLogger

- **Purpose**: Core logging engine
- **Responsibilities**:
  - Handles actual `console.log()` calls
  - Manages log levels (DEBUG, INFO, WARN, ERROR)
  - Implements the `shouldLog()` decision logic
  - Formats log output with timestamps and service tags

#### ServiceLogger

- **Purpose**: Convenience wrapper for service-specific logging
- **Responsibilities**:
  - Adds service context automatically
  - Provides method chaining: `logger.dashboard().debug()`
  - Prevents repetitive service string passing
  - Delegates to CoreLogger with enhanced context

### 2. Flag Hierarchy

The logging system uses a three-tier flag hierarchy:

```
ENABLE_LOGGING (Master Flag)
    ↓
Context Flags (Frontend/Backend)
    ↓
Service Flags (Specific Services)
    ↓
Actual Log Output
```

#### Master Flag (Meta)

- `ENABLE_LOGGING` - Global kill switch for all logging
- **Purpose**: Emergency stop for all logging across the application

#### Context Flags (Meta)

- `ENABLE_FRONTEND_LOGGING` - Controls `:fe` context (frontend components)
- `ENABLE_BACKEND_LOGGING` - Controls `:be` context (backend APIs)
- **Purpose**: Separate frontend vs backend logging concerns

#### Service Flags (Specific)

- `ENABLE_DASHBOARD_LOGGING` - Controls `dashboard:` service
- `ENABLE_UPLOAD_LOGGING` - Controls `upload:` service
- `ENABLE_DATABASE_LOGGING` - Controls `database:` service
- `ENABLE_AUTH_LOGGING` - Controls `auth:` service
- And many more...
- **Purpose**: Fine-grained control per service/feature

## Logging Flow

### Step-by-Step Process

Let's trace a typical logging call:

```typescript
logger.dashboard().debug("User clicked button", { userId: "123" });
```

#### Step 1: ServiceLogger Creation

```typescript
logger.dashboard();
// Creates: ServiceLogger('dashboard', 'fe', parentLogger)
// Returns: ServiceLogger instance with service context
```

#### Step 2: ServiceLogger.debug() Execution

```typescript
// ServiceLogger.debug() internally calls:
this.parentLogger.debug("User clicked button", "dashboard:fe", { userId: "123" });
// Delegates to CoreLogger with enhanced context
```

#### Step 3: CoreLogger.shouldLog() Decision

```typescript
shouldLog(LogLevel.DEBUG, 'dashboard:fe') {
  // 1. Check master switch
  if (!config.ENABLE_LOGGING) return false; // ❌ BLOCKED

  // 2. Check log level threshold
  if (LogLevel.DEBUG < this.level) return false; // ❌ BLOCKED

  // 3. Parse service:context format
  const [serviceName, context] = 'dashboard:fe'.split(':');
  // serviceName = 'dashboard', context = 'fe'

  // 4. Check service-specific flag
  let serviceEnabled = true; // Default for unknown services
  switch (serviceName) {
    case 'dashboard':
      serviceEnabled = config.ENABLE_DASHBOARD_LOGGING;
      break;
    case 'upload':
      serviceEnabled = config.ENABLE_UPLOAD_LOGGING;
      break;
    // ... other services
  }

  // 5. Check context flag (frontend vs backend)
  const contextEnabled = context === 'fe' ?
    config.ENABLE_FRONTEND_LOGGING :  // ✅ CHECKED
    config.ENABLE_BACKEND_LOGGING;

  // 6. Final decision (AND logic)
  return serviceEnabled && contextEnabled; // ✅ ALLOWED
}
```

#### Step 4: Log Output (if allowed)

```typescript
// If shouldLog() returns true:
const prefix = this.formatPrefix("DEBUG", "dashboard:fe");
// Result: "[2025-01-07T10:30:00.000Z] DEBUG [dashboard:fe]"
console.debug(prefix, "User clicked button", { userId: "123" });
```

## Flag Combination Logic

All flags work as **AND gates** - all conditions must be true for logging to occur:

```
ENABLE_LOGGING (Master)
    AND
ENABLE_FRONTEND_LOGGING (Context)
    AND
ENABLE_DASHBOARD_LOGGING (Service)
    =
LOG OUTPUT
```

## Real-World Examples

### Example 1: Frontend Dashboard Logging

```typescript
// User clicks button in dashboard component
logger.dashboard().info("Button clicked", { buttonId: "save" });

// Flow:
// 1. ServiceLogger('dashboard', 'fe') created
// 2. Calls CoreLogger.info('Button clicked', 'dashboard:fe', { buttonId: 'save' })
// 3. Checks: ENABLE_LOGGING && ENABLE_FRONTEND_LOGGING && ENABLE_DASHBOARD_LOGGING
// 4. Output: "[2025-01-07T10:30:00.000Z] INFO [dashboard:fe] Button clicked { buttonId: 'save' }"
```

### Example 2: Backend API Logging

```typescript
// API route processes database query
logger.database().debug("Query executed", { query: "SELECT * FROM users" });

// Flow:
// 1. ServiceLogger('database', 'be') created
// 2. Calls CoreLogger.debug('Query executed', 'database:be', { query: 'SELECT * FROM users' })
// 3. Checks: ENABLE_LOGGING && ENABLE_BACKEND_LOGGING && ENABLE_DATABASE_LOGGING
// 4. Output: "[2025-01-07T10:30:00.000Z] DEBUG [database:be] Query executed { query: 'SELECT * FROM users' }"
```

### Example 3: Upload Service Warning

```typescript
// File upload fails
logger.upload().warn("Upload failed", { error: "File too large" });

// Flow:
// 1. ServiceLogger('upload', 'be') created
// 2. Calls CoreLogger.warn('Upload failed', 'upload:be', { error: 'File too large' })
// 3. Checks: ENABLE_LOGGING && ENABLE_BACKEND_LOGGING && ENABLE_UPLOAD_LOGGING
// 4. Output: "[2025-01-07T10:30:00.000Z] WARN [upload:be] Upload failed { error: 'File too large' }"
```

## Configuration System

### Runtime Configuration

The logger system supports runtime configuration through localStorage:

```typescript
// Three-state toggle system:
type ToggleState = "not-set" | "enabled" | "disabled";

// Resolution logic:
function resolveToggleState(uiState: string | undefined, defaultValue: boolean): boolean {
  if (uiState === "enabled") return true;
  if (uiState === "disabled") return false;
  return defaultValue; // 'not-set' uses hardcoded default
}
```

### Configuration Priority

1. **UI Override** - User settings in localStorage
2. **Hardcoded Defaults** - Fallback values in `logger.ts`
3. **Server-Side** - Always uses hardcoded defaults (no localStorage)

## Service Context Format

The logger uses a `service:context` format for identifying log sources:

- `dashboard:fe` - Frontend dashboard components
- `database:be` - Backend database operations
- `upload:be` - Backend upload processing
- `auth:be` - Backend authentication
- `memory-processing:fe` - Frontend memory processing

## Benefits of This Architecture

### 1. Hierarchical Control

- Master switch for emergency stops
- Context separation (frontend vs backend)
- Service-specific granular control

### 2. Performance Optimization

- Early exits prevent unnecessary processing
- Flag checks happen before expensive operations
- Minimal overhead when logging is disabled

### 3. Developer Experience

- Clean API: `logger.dashboard().debug()`
- Automatic context injection
- Consistent formatting across all logs

### 4. Flexibility

- Runtime configuration changes
- Persistent settings via localStorage
- Fallback to sensible defaults

### 5. Maintainability

- Clear separation of concerns
- Easy to add new services
- Centralized configuration management

## Usage Patterns

### Basic Logging

```typescript
import { logger } from "@/lib/logger";

// Simple logging
logger.info("Application started");
logger.error("Database connection failed", error);

// Service-specific logging
logger.dashboard().debug("Component rendered", { props });
logger.upload().warn("File size exceeded limit", { size });
```

### Configuration

```typescript
// Runtime configuration (via UI)
// Master switch: ENABLE_LOGGING = 'enabled'
// Frontend logging: ENABLE_FRONTEND_LOGGING = 'disabled'
// Dashboard logging: ENABLE_DASHBOARD_LOGGING = 'enabled'

// Result: Only backend dashboard logs will appear
```

This architecture provides a robust, flexible, and performant logging system that scales with the application's needs while maintaining clear control over log output.
