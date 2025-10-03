# ICP Utilities Folder Duplication Issue

## 📋 **Issue Summary**

**Status**: 🔴 **OPEN** - Requires architectural decision and consolidation

**Problem**: There's unnecessary duplication between `src/ic/` and `src/lib/` folders for ICP-related utilities, leading to confusion about where to place new functionality and making maintenance difficult.

## 🔍 **Current State Analysis**

### **`src/ic/` folder** (ICP-specific utilities)

- `actor-factory.ts` - Generic actor creation
- `agent.ts` - Agent creation with caching
- `backend.ts` - Backend actor creation
- `ii.ts` - Internet Identity authentication

### **`src/lib/` folder** (General utilities + some ICP)

- `ii-client.ts` - Internet Identity client utilities
- `ii-coauth-guard.ts` - II co-authentication guard
- `ii-coauth-ttl.ts` - II co-authentication TTL
- `ii-nonce.ts` - II nonce utilities
- `server-actor.ts` - Server-side actor creation
- `icp-upload-mapper.ts` - ICP upload type mapping

## 🚨 **Identified Duplications**

1. **`src/ic/ii.ts`** vs **`src/lib/ii-client.ts`** - Both handle II authentication
2. **`src/ic/backend.ts`** vs **`src/lib/server-actor.ts`** - Both create actors
3. **`src/ic/agent.ts`** vs **`src/lib/server-actor.ts`** - Both handle agent creation

## 🎯 **Proposed Solution**

**Consolidate into `src/lib/`** and remove the `src/ic/` folder:

### **Keep in `src/lib/`:**

- `ii-client.ts` - Main II authentication utilities
- `ii-coauth-guard.ts` - II co-authentication guard
- `ii-coauth-ttl.ts` - II co-authentication TTL
- `ii-nonce.ts` - II nonce utilities
- `server-actor.ts` - Server-side actor creation
- `icp-upload-mapper.ts` - ICP upload type mapping

### **Remove `src/ic/` folder:**

- Move any unique functionality from `src/ic/` to `src/lib/`
- Update imports across the codebase

## 🔧 **Benefits of Consolidation**

1. **Single source of truth** for ICP utilities
2. **Clearer architecture** - all utilities in `lib/`
3. **Easier maintenance** - no duplicate functionality
4. **Better organization** - follows the established pattern

## 📋 **Action Plan**

1. **Audit `src/ic/`** - Check if there's any unique functionality
2. **Move unique code** to `src/lib/` if needed
3. **Update all imports** from `@/ic/*` to `@/lib/*`
4. **Remove `src/ic/` folder**
5. **Test that everything still works**

## 🚧 **Impact Assessment**

- **Files to update**: All files importing from `@/ic/*`
- **Risk level**: Medium - requires careful import updates
- **Testing required**: Full ICP functionality testing
- **Estimated effort**: 2-3 hours for consolidation + testing

## 📝 **Notes**

- This is an architectural cleanup issue
- Should be done when there's time for proper testing
- Consider creating a migration script for import updates
- Document the new structure after consolidation
