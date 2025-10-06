# Hosting Preferences Toggle Logic Fix

**Priority**: High  
**Type**: Bug Fix  
**Assigned To**: Development Team  
**Created**: 2025-01-06  
**Status**: Open

## 🐛 **Problem Description**

The hosting preferences toggles in the settings page have incorrect logic. Currently, when a user has Web2 backend enabled and then enables Web3 backend, the Web2 backend gets turned off. This is wrong behavior.

## 📋 **Expected Behavior**

Users should be able to have **both Web2 and Web3 stacks enabled simultaneously**:

1. **Web2 Backend + Web2 Database**: Always together (if one is on, both are on)
2. **Web3 Backend + Web3 Database**: Always together (if one is on, both are on)
3. **Both stacks can coexist**: Users can have Web2 + Web3 running at the same time
4. **At least one stack must be enabled**: Users cannot turn off both Web2 and Web3 completely

## 📍 **Scope & Usage**

**These hosting preferences are persisted in the database and read via API**:

- **Database**: `user_hosting_preferences` table stores `web2_enabled` and `web3_enabled` booleans
- **API**: `/api/hosting-preferences` endpoint reads/writes these values
- **Settings Page**: Users configure which hosting stacks are enabled (reads from API)
- **Dashboard**: Uses these preferences to determine which data sources to show/use
- **Memory Operations**: Backend services use these preferences to route requests to appropriate hosting providers

**The hosting preferences do NOT control:**

- Database view switching in dashboard (that's local UI state)
- Memory creation/deletion logic (that's handled by backend services)
- User authentication (that's separate from hosting preferences)

## 🔧 **Current Incorrect Behavior**

```typescript
// Current logic in settings page
onCheckedChange: (checked) => {
  if (checked) {
    // This is correct - enables both backend and database
    updatePreferences.mutate({
      backendHosting: "vercel",
      databaseHosting: ["neon"],
    });
  } else {
    // This is WRONG - it switches to ICP instead of allowing both
    updatePreferences.mutate({ backendHosting: "icp" });
  }
};
```

## ✅ **Required Fix**

The toggles should work as **independent switches** that can both be ON simultaneously:

- **Web2 Backend Toggle**: ON/OFF (independent)
- **Web2 Database Toggle**: ON/OFF (but synced with Web2 Backend)
- **Web3 Backend Toggle**: ON/OFF (independent)
- **Web3 Database Toggle**: ON/OFF (but synced with Web3 Backend)

## 🎯 **Acceptance Criteria**

- [ ] Web2 backend can be enabled independently of Web3 backend
- [ ] Web3 backend can be enabled independently of Web2 backend
- [ ] Both Web2 and Web3 can be enabled simultaneously
- [ ] Web2 backend and Web2 database are always in sync
- [ ] Web3 backend and Web3 database are always in sync
- [ ] At least one stack (Web2 or Web3) must always be enabled
- [ ] Users cannot turn off both stacks completely

## 🔍 **Technical Details**

**File**: `src/nextjs/src/app/[lang]/user/settings/page.tsx`

**Current Issue**: The `onCheckedChange` handlers are using radio button logic (only one can be selected) instead of checkbox logic (multiple can be selected).

**Required Change**: Update the hosting preferences data structure and toggle logic to support multiple simultaneous selections.

## 🚀 **Recommended Implementation (Tech Lead Approved)**

### **Data Model**

Keep exactly one persisted source of truth for the stacks:

```typescript
// Persisted (DB/API)
type HostingStacks = {
  web2Enabled: boolean; // Web2 = Vercel + Neon
  web3Enabled: boolean; // Web3 = ICP backend + ICP DB
};
```

Everything else (`backendHosting`, `databaseHosting`) becomes **derived**:

```typescript
// Derived (UI/adapters only)
const backendHosting = [state.web2Enabled ? "vercel" : null, state.web3Enabled ? "icp" : null].filter(Boolean) as Array<
  "vercel" | "icp"
>;

const databaseHosting = [state.web2Enabled ? "neon" : null, state.web3Enabled ? "icp" : null].filter(Boolean) as Array<
  "neon" | "icp"
>;
```

### **UI Toggle Logic (Checkbox Semantics)**

Acceptance rule: **at least one** stack must stay enabled.

```typescript
function toggleWeb2(next: boolean) {
  const newState = { web2Enabled: next, web3Enabled: state.web3Enabled };
  if (!newState.web2Enabled && !newState.web3Enabled) {
    toast({ title: "One stack required", description: "Keep Web2 or Web3 enabled." });
    return;
  }
  saveStacks(newState);
}

function toggleWeb3(next: boolean) {
  const newState = { web2Enabled: state.web2Enabled, web3Enabled: next };
  if (!newState.web2Enabled && !newState.web3Enabled) {
    toast({ title: "One stack required", description: "Keep Web2 or Web3 enabled." });
    return;
  }
  saveStacks(newState);
}
```

Bind these to independent checkboxes:

```tsx
<Switch checked={state.web2Enabled} onCheckedChange={toggleWeb2} />
<Switch checked={state.web3Enabled} onCheckedChange={toggleWeb3} />
```

### **Server / DB Changes**

**No database changes needed!** We can implement this using the existing structure:

- **Use existing fields**: `backendHosting` and `databaseHosting`
- **Derive boolean states** in the frontend from existing data
- **Convert back to existing format** when saving

### **Frontend Implementation (No DB Changes)**

**The sync constraint is only a frontend UX rule** - the backend can handle any combination of values.

```typescript
// Derive boolean states from existing data
function getWeb2Enabled(prefs: HostingPreferences): boolean {
  return prefs.backendHosting === "vercel" || prefs.databaseHosting.includes("neon");
}

function getWeb3Enabled(prefs: HostingPreferences): boolean {
  return prefs.backendHosting === "icp" || prefs.databaseHosting.includes("icp");
}

// Convert boolean states back to existing format
// Frontend enforces sync: Web2 backend always comes with Web2 database, etc.
function saveStacks(web2Enabled: boolean, web3Enabled: boolean) {
  const backendHosting = web2Enabled ? "vercel" : "icp";
  const databaseHosting = [...(web2Enabled ? ["neon"] : []), ...(web3Enabled ? ["icp"] : [])];

  updatePreferences.mutate({ backendHosting, databaseHosting });
}
```

**Key Point**: The backend doesn't enforce sync constraints - it accepts any valid combination. The frontend just ensures a good UX by keeping Web2 backend + Web2 database together, and Web3 backend + Web3 database together.

### **What to Remove/Fix**

- Delete the "switch to ICP if Web2 unchecked" branch. That's radio logic.
- Stop persisting `current*View` or any active tab; these are ephemeral UI states.
- Update `updatePreferences.mutate(...)` to send only `{ web2Enabled, web3Enabled }`.

## ✅ **Validation Cases (Must Pass)**

- Turn **ON** Web2 while Web3 is **ON** → both **ON** ✅
- Turn **OFF** Web2 while Web3 is **ON** → Web3 stays **ON** ✅
- Turn **OFF** Web3 while Web2 is **ON** → Web2 stays **ON** ✅
- Try to turn **OFF** the last enabled stack → **blocked with toast** ✅
- Reload page → both toggles reflect persisted booleans ✅
- Derived `databaseHosting` shows `['neon','icp']` only when both toggles are ON ✅

## 🎯 **Benefits of This Approach**

- **One boolean per stack** in DB/API - simple and canonical
- **Independent checkboxes** with "at least one on" guard
- Backend/DB arrays remain **derived**, not stored as primary truth
- **Zero chance of Web2 ↔ Web3 de-sync**
- **Impossible to desync** backend and database within each stack
- **Dead simple to validate** and reason about

## 🔗 **Related Issues**

- [Dashboard ICP/Neon Database Switching](./dashboard-icp-neon-database-switching-todo.md) - Original implementation
- [Advanced Database Toggle Implementation](./dashboard-icp-neon-database-switching-todo.md) - Current work

---

**Last Updated**: 2025-01-06  
**Status**: Approved Implementation Plan - Ready for Development
