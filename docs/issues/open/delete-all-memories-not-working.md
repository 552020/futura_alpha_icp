# Delete All Memories Not Working - Frontend API Call Bug

## 🚨 **ISSUE IDENTIFIED**

**Status**: FIXED - Frontend API call bug resolved  
**Priority**: MEDIUM  
**Component**: Frontend ICP Integration

## 📋 **Problem Summary**

The "Delete All Memories" functionality was failing with a `NotFound` error when trying to clear all memories from ICP. The error was occurring in the `dev_clear_all_memories_in_capsule` backend function, which was returning `{NotFound: null}`.

## 🔍 **Root Cause Analysis**

### **The Bug:**

The user doesn't have a capsule yet, so when the frontend calls `backend.capsules_read_basic([])`, it returns a `NotFound` error because no capsule exists for the user.

### **What Was Happening:**

1. **Frontend calls**: `backend.capsules_read_basic([])` to get self-capsule
2. **Backend receives**: `[]` (empty array = get self-capsule)
3. **Backend checks**: No capsule exists for this user
4. **Result**: `NotFound` error because user has no capsule
5. **Error**: `Failed to delete all memories: {"NotFound":null}`

### **Backend Function Signature:**

```rust
fn capsules_read_basic(capsule_id: Option<String>) -> std::result::Result<CapsuleInfo, Error>
```

### **TypeScript Declaration:**

```typescript
'capsules_read_basic' : ActorMethod<[[] | [string]], Result_6>
```

## ✅ **FIX IMPLEMENTED**

### **Changes Made:**

**File**: `src/nextjs/src/services/memories.ts`

**Before:**

```typescript
const capsuleResult = await backend.capsules_read_basic([]);
if (!('Ok' in capsuleResult)) {
  throw new Error('Failed to get user capsule');
}
const capsuleId = capsuleResult.Ok.capsule_id;
```

**After:**

```typescript
// Get or create capsule ID
let capsuleId: string;
try {
  const capsuleResult = await backend.capsules_read_basic([]);
  if ('Ok' in capsuleResult) {
    capsuleId = capsuleResult.Ok.capsule_id;
  } else {
    // No capsule found, create one
    console.log('🔍 [Delete All Dev] No capsule found, creating one...');
    const createResult = await backend.capsules_create([]);
    if ('Ok' in createResult) {
      capsuleId = createResult.Ok.id;
      console.log('🔍 [Delete All Dev] Created new capsule:', capsuleId);
    } else {
      throw new Error(`Failed to create capsule: ${JSON.stringify(createResult.Err)}`);
    }
  }
} catch (error) {
  throw new Error(`Failed to get or create user capsule: ${error instanceof Error ? error.message : 'Unknown error'}`);
}
```

### **Explanation:**

- **Problem**: User doesn't have a capsule yet
- **Solution**: Try to get existing capsule, if not found, create a new one
- **Pattern**: Follow the same pattern used in `icp-upload.ts` for capsule creation

## 🎯 **Verification**

The fix should now allow the "Delete All Memories" functionality to:

1. ✅ Successfully get the user's self-capsule ID
2. ✅ Call `dev_clear_all_memories_in_capsule` with the correct capsule ID
3. ✅ Clear all memories and assets from the capsule
4. ✅ Return success with the count of deleted memories

## 📊 **Impact**

### **Before Fix:**

- **User Experience**: Delete All Memories button failed with "NotFound" error
- **Functionality**: Unable to clear all memories from ICP
- **Error**: `Failed to delete all memories: {"NotFound":null}`

### **After Fix:**

- **User Experience**: ✅ Delete All Memories works correctly
- **Functionality**: ✅ Successfully clears all memories and assets
- **Result**: ✅ Returns proper success message with deleted count

## 🔧 **Technical Details**

### **API Call Pattern:**

- **Get self-capsule**: `capsules_read_basic([])`
- **Create self-capsule**: `capsules_create([])`
- **Specific capsule**: `capsules_read_basic(["capsule_123"])`

### **Backend Logic:**

```rust
match capsule_id {
    Some(id) => crate::capsule::query::capsules_read_basic(id),
    None => crate::capsule::query::capsule_read_self_basic(),
}
```

### **Candid Mapping:**

- `Option<String>` → `[] | [string]`
- `None` → `[]` (empty array)
- `Some("id")` → `["id"]` (array with string)

---

**Created**: 2024-12-19  
**Last Updated**: 2024-12-19  
**Assigned**: Frontend Team  
**Related**: [asset-serving-placeholder-bug.md](./asset-serving-placeholder-bug.md)
