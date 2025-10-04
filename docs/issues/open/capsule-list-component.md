# Capsule List Component

## Problem Statement

We need a component to display capsules in a list/table format for users to view and manage multiple capsules. This component should show capsule information in a structured, easy-to-read format.

## Current State

- ✅ **Single capsule view** - `CapsuleInfo` component shows one capsule
- ✅ **Capsule creation** - `CreateCapsuleModal` for creating new capsules
- ❌ **Capsule list view** - No way to see all user's capsules

## Backend Types Reference

The capsule types are already documented in our backend API documentation. See:

- `docs/backend-api-documentation.md` - Complete backend API reference
- `src/nextjs/src/types/capsule.ts` - Frontend type definitions

### Key Backend Types (from `backend.did`):

```typescript
// Core capsule types
export interface Capsule {
  id: string;
  subject: PersonRef;
  owners: Array<[PersonRef, OwnerState]>;
  controllers: Array<[PersonRef, ControllerState]>;
  created_at: bigint;
  updated_at: bigint;
  bound_to_neon: boolean;
  galleries: Array<GalleryHeader>;
  memories: Array<MemoryHeader>;
  connections: Array<[PersonRef, Connection]>;
}

export interface CapsuleInfo {
  capsule_id: string;
  subject: PersonRef;
  is_owner: boolean;
  is_controller: boolean;
  is_self_capsule: boolean;
  created_at: bigint;
  updated_at: bigint;
  bound_to_neon: boolean;
  gallery_count: bigint;
  memory_count: bigint;
  connection_count: bigint;
}

export type PersonRef = { Opaque: string } | { Principal: Principal };
```

## Proposed Solution

Create a `CapsuleList` component that displays capsules in a table format with the following features:

### Component Structure

```typescript
interface CapsuleListProps {
  capsules: CapsuleInfo[];
  isLoading: boolean;
  error?: CapsuleError;
  onCapsuleSelect?: (capsuleId: string) => void;
  onCapsuleCreate?: () => void;
}
```

### Table Columns (Proposed)

| Column          | Data Source                     | Description                                                |
| --------------- | ------------------------------- | ---------------------------------------------------------- |
| **Subject**     | `subject` + `is_self_capsule`   | "You" for self-capsule, person/entity name for others      |
| **Role**        | `is_owner`, `is_controller`     | "Owner", "Controller", "Both" (owner is always controller) |
| **Web2 Link**   | `bound_to_neon`                 | "Connected" or "ICP Only" (linked to web2 account)         |
| **Storage**     | `canister_type` (hardcoded)     | "Shared" (all capsules for now)                            |
| **Memories**    | `memory_count`                  | Number of memories                                         |
| **Galleries**   | `gallery_count`                 | Number of galleries                                        |
| **Connections** | `connection_count`              | Number of connections                                      |
| **Space**       | `storage_used`, `storage_limit` | Used/Total storage (e.g., "2.5GB / 10GB")                  |
| **Lifetime**    | `expires_at`                    | Expiration year (e.g., "2029")                             |
| **Actions**     | -                               | View, Edit, Delete buttons                                 |

### UI Design

- **Table format** with sortable columns
- **Responsive design** for mobile/desktop
- **Loading states** and error handling
- **Empty state** with "Create Capsule" button
- **Row actions** for each capsule

## Implementation Plan

### Phase 1: Basic List Component

- [ ] **1.1** Create `CapsuleList` component structure
- [ ] **1.2** Implement table with basic columns
- [ ] **1.3** Add loading and error states
- [ ] **1.4** Add empty state with create button

### Phase 2: Enhanced Features

- [ ] **2.1** Add row actions (view, edit, delete)
- [ ] **2.2** Add basic navigation
- [ ] **2.3** Add responsive design improvements

### Phase 3: Integration

- [ ] **3.1** Integrate with `CapsuleInfo` component
- [ ] **3.2** Add navigation between list and detail views
- [ ] **3.3** Test complete user flow

### Phase 4: Capsule Detail Component

- [ ] **4.1** Create `CapsuleDetail` component for viewing individual capsules
- [ ] **4.2** Add navigation from list to detail view
- [ ] **4.3** Implement detailed capsule information display
- [ ] **4.4** Add edit/delete functionality in detail view

## Post-MVP Features

**Advanced features** (sorting, filtering, search, bulk operations) are documented in:

- `docs/issues/open/capsule-list-advanced-features.md`

## Discussion Points

### New Backend Fields Needed

**Storage Information:**

- `storage_used: bigint` - Current storage usage in bytes
- `storage_limit: bigint` - Maximum storage allowed in bytes
- Display format: "2.5GB / 10GB" or "25% used"

**Lifetime Information:**

- `expires_at: bigint` - When the capsule expires (timestamp)
- Display format: "5 years remaining" or "Expires in 2 months"
- Could be calculated from `created_at + lifetime_duration`

### Column Name Suggestions

**Storage Column:**

- "Storage" - Clear and simple
- "Canister" - Technical but accurate
- "Type" - Generic but understandable
- "Location" - Where the capsule is stored

**My recommendation: "Storage"** - Most user-friendly

### Backend Questions

**Missing fields that need to be added to backend:**

1. **Storage tracking:** `storage_used` and `storage_limit` fields
2. **Lifetime/expiration:** `expires_at` field for capsule expiration
3. **Canister type:** `canister_type` field to distinguish independent vs shared

**Action:** Create separate issue `capsule-backend-add-missing-information` for backend changes

### Missing Backend Information

**Storage and Lifetime Tracking:**

- **Current status:** Not available in backend types
- **Missing fields:**
  - `storage_used: bigint` - Current storage usage in bytes
  - `storage_limit: bigint` - Maximum storage allowed in bytes
  - `expires_at: bigint` - When the capsule expires (timestamp)
- **Backend change needed:** Add storage and lifetime tracking to `Capsule` type
- **For now:** All capsules show "Shared" storage type, hide Space and Lifetime columns

**Implementation approach:**

- **Phase 1:** Implement list with hardcoded "Shared" storage and placeholder values for Space/Lifetime
- **Phase 2:** Add backend fields for storage and lifetime tracking
- **Phase 3:** Update frontend to use real data

**Display format:**

- **Space:** "2.5GB / 10GB" (used/total)
- **Lifetime:** "2029" (expiration year only)

### UI Considerations

**Space Column:**

- Show progress bar for visual representation
- Color coding (green/yellow/red) for usage levels
- Tooltip with exact bytes

**Lifetime Column:**

- Show expiration year only (e.g., "2029")
- Color coding for urgency (green/yellow/red)

## Technical Considerations

### Data Loading

- Use `listCapsules()` service function
- Implement pagination for large capsule lists
- Cache capsule data to avoid repeated API calls

### State Management

- Extend existing `CapsulesState` interface
- Use `useAuthenticatedActor` for API calls
- Handle loading and error states consistently

### UI Components

- Use shadcn/ui Table component
- Follow existing design patterns
- Ensure accessibility and responsive design

## Success Criteria

- ✅ **Display capsule list** in table format
- ✅ **Show key capsule information** (subject, type, counts, dates)
- ✅ **Handle loading and error states** gracefully
- ✅ **Provide actions** for each capsule (view, edit, delete)
- ✅ **Responsive design** works on mobile and desktop
- ✅ **Integration** with existing capsule components

## Dependencies

- `src/nextjs/src/services/capsule.ts` - `listCapsules()` function
- `src/nextjs/src/types/capsule.ts` - Type definitions
- `src/nextjs/src/components/ui/table.tsx` - shadcn Table component
- `src/nextjs/src/hooks/use-authenticated-actor.ts` - Authentication

## Files to Create

- `src/nextjs/src/components/icp/capsule-list.tsx` - Main list component
- `src/nextjs/src/components/icp/capsule-list-item.tsx` - Individual row component
- `src/nextjs/src/components/icp/capsule-list-actions.tsx` - Row actions component

## Future Enhancements

- **Bulk operations** (select multiple capsules)
- **Advanced filtering** (by date, type, owner)
- **Export functionality** (CSV, PDF)
- **Real-time updates** when capsules change
- **Drag and drop** for reordering
