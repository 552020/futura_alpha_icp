# Manual Merge Strategy: Gallery Duplication Issue

## **Problem Statement**

We need to merge `main` branch into our feature branch `552020/icp-396-restore-global-order-only-one-will-survive` while preserving both gallery implementations:

1. **Old Gallery State** (current branch) - with Store Forever functionality
2. **New Gallery State** (from main) - with selection mode functionality

## **Current Situation**

- **Current Branch**: `552020/icp-396-restore-global-order-only-one-will-survive`

  - Has old gallery implementation with Store Forever functionality
  - Gallery components: `gallery-card.tsx`, `gallery-list.tsx`
  - Complex Store Forever button with ICP integration

- **Main Branch**:
  - Has new gallery implementation with selection mode
  - Removed `gallery-card.tsx` and `gallery-list.tsx`
  - Added new components: `gallery-photo-grid.tsx`, `gallery-selection-bar.tsx`, etc.

## **Goal**

Merge main while keeping **both gallery implementations** side by side, allowing users to choose between:

- **Old Gallery**: Store Forever functionality
- **New Gallery**: Selection mode functionality

## **Proposed Strategy**

### **Option 1: Manual Merge with --no-commit (Recommended)**

```bash
git merge main --no-commit
```

**Benefits:**

- Full control over each conflict
- Can keep both implementations
- Review each change before committing
- No automatic decisions by Git

**Process:**

1. Start merge without committing
2. Manually resolve each conflict
3. Keep both gallery implementations
4. Rename components to avoid conflicts (e.g., `gallery-card-v2.tsx`)
5. Create routing logic to choose between implementations

### **Option 2: File-by-file Manual Merge**

```bash
# See what files changed in main
git diff --name-only main

# For each file, manually merge
git checkout main -- <specific-file>
# Then manually edit to keep both versions
```

### **Option 3: Cherry-pick Specific Changes**

```bash
# See what commits are in main
git log main --oneline

# Cherry-pick only the changes you want
git cherry-pick <commit-hash>
```

## **Implementation Plan**

### **Phase 1: Conflict Resolution**

1. Use `git merge main --no-commit`
2. For each conflict, decide:
   - Keep old implementation
   - Keep new implementation
   - Keep both (rename one)
   - Create hybrid solution

### **Phase 2: Component Renaming**

- `gallery-card.tsx` → `gallery-card-legacy.tsx`
- `gallery-list.tsx` → `gallery-list-legacy.tsx`
- New components keep original names

### **Phase 3: Routing Logic**

Create feature flag or user preference to choose between:

- **Legacy Gallery**: Store Forever functionality
- **Modern Gallery**: Selection mode functionality

### **Phase 4: Testing**

- Test both gallery implementations
- Ensure no conflicts between them
- Verify build works with both

## **Expected Conflicts**

1. **Gallery Components**: `gallery-card.tsx`, `gallery-list.tsx`
2. **Gallery Pages**: `gallery/[id]/page.tsx`
3. **Import Statements**: Components importing deleted files
4. **Type Definitions**: Gallery types and interfaces

## **Success Criteria**

- ✅ Both gallery implementations work
- ✅ No build errors
- ✅ Users can choose between implementations
- ✅ All existing functionality preserved
- ✅ New functionality available

## **Risks**

- **Complexity**: Maintaining two gallery implementations
- **Bundle Size**: Larger JavaScript bundle
- **Maintenance**: More code to maintain
- **User Confusion**: Two different gallery experiences

## **Mitigation**

- Clear documentation of differences
- Feature flags for easy switching
- Gradual migration path
- User testing to validate approach

## **Next Steps**

1. Execute manual merge strategy
2. Resolve conflicts one by one
3. Test both implementations
4. Document the differences
5. Create migration plan for users

---

**Created**: 2024-10-01  
**Status**: Open  
**Priority**: High  
**Assignee**: TBD
