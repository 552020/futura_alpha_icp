# Logging Guidelines

## Two Logger System

We have two loggers with different purposes:

### 🪶 tinyLogger (Default)

**File:** `src/lib/logger/tiny-logger.ts`

**When to use:** Everywhere by default. This is our primary logging solution.

**Features:**

- Zero dependencies
- Tag-based system (everything is a tag)
- Simple API: `tinyLogger(message, { tags, data })`
- Auto-detects log levels from tags
- Optional filtering with `setLoggerFilter()` and `toggleLogger()`

**Usage:**

```typescript
import { tinyLogger } from "@/lib/logger/tiny-logger";
// Or import both loggers together:
// import { tinyLogger, fatLogger } from "@/lib/logger";

// Basic usage
tinyLogger("User clicked upload", { tags: ["debug", "frontend", "dashboard", "upload"] });

// With data
tinyLogger("Upload failed", {
  tags: ["warn", "backend", "upload", "error"],
  data: { error: "File too large" },
});
```

### 🍔 fatLogger (Refactored/Experimental)

**File:** `src/lib/logger/fat-logger/`

**When to use:** Only for experimental features or when you need the complex hierarchical system.

**Features:**

- Clean modular architecture (LogEngine, ServiceLogger, FatLogger)
- No circular dependencies
- Service-specific logging with context awareness
- Runtime configuration and filtering
- Extensible for future structured logging

**Usage:**

```typescript
import { fatLogger } from "@/lib/logger/fat-logger";
// Or import both loggers together:
// import { tinyLogger, fatLogger } from "@/lib/logger";

// Service-specific logging
const log = fatLogger.service("upload", "fe");
log.debug("Dashboard rendered", { userId: "123" }, ["ui", "interaction"]);

// Direct logging with convenience methods
fatLogger.info("File uploaded", "be", { size: "2MB" });
fatLogger.debug("Dashboard render started", "fe", { page: "main" }, ["dashboard"]);
```

## Decision Tree

```
Need logging?
├── Use tinyLogger (99% of cases)
└── Need complex hierarchical logging?
    └── Use fatLogger (experimental only)
```

## Migration

- **New code:** Always use `tinyLogger`
- **Existing code:** Gradually migrate from `fatLogger` to `tinyLogger`
- **Settings page:** Both loggers can be tested at `/user/settings/logger`

## Testing

Visit `/user/settings/logger` to test both loggers with different configurations and filters.
