# ICP-396: Manual Merge Strategy for Gallery Duplication

## Problem Statement

We need to merge the latest main branch changes into our `552020/icp-396-restore-global-order-only-one-will-survive` branch while preserving the old gallery functionality (including "Store Forever" button and gallery components). A standard merge would lose our old gallery implementation.

## Current Situation

### Branches Created
- **`552020/icp-396-restore-global-order-only-one-will-survive`**: Pre-merge state with old gallery functionality
- **`snapshot/icp-396-main-state-2025-01-10`**: Snapshot of main branch state for reference

### Key Differences
- **213 files** differ between our pre-merge state and main
- Main has evolved significantly with new features:
  - New gallery pages and components
  - Enhanced memory management
  - New API endpoints
  - Database schema changes
  - New authentication flows

## Goal

Create a hybrid implementation that:
1. **Preserves** old gallery functionality (Store Forever, gallery-card, gallery-list)
2. **Integrates** new main features (dashboard, memory management, etc.)
3. **Allows** side-by-side comparison of old vs new gallery implementations
4. **Maintains** clean commit history

## Proposed Strategy

### Phase 1: Analysis (COMPLETED)
- ✅ Identified pre-merge state: commit `8c23047`
- ✅ Created clean branch from pre-merge state
- ✅ Created main snapshot for reference
- ✅ Documented 213 file differences

### Phase 2: Controlled Integration
Use file-by-file merge approach:

```bash
# For each file we want to integrate from main:
git checkout main -- path/to/file
# Review changes
git add -p path/to/file
git commit -m "Manual integrate: path/to/file from main"
```

### Phase 3: Gallery Duplication Strategy
1. **Keep old gallery components** in their current state
2. **Add new gallery components** alongside (with different names/paths)
3. **Create routing** to access both implementations
4. **Document differences** between old and new approaches

## Implementation Plan

### Priority 1: Core Infrastructure
- [ ] Database schema updates
- [ ] Authentication enhancements
- [ ] Core API improvements

### Priority 2: Memory Management
- [ ] New memory upload flows
- [ ] Enhanced file processing
- [ ] Storage management improvements

### Priority 3: Gallery Duplication
- [ ] Preserve old gallery components
- [ ] Integrate new gallery components
- [ ] Create comparison interface
- [ ] Document feature differences

## Files to Preserve (Old Gallery)
- `src/components/galleries/gallery-card.tsx`
- `src/components/galleries/gallery-list.tsx`
- `src/app/[lang]/gallery/[id]/page.tsx` (with Store Forever button)

## Files to Integrate (New Features)
- All new API endpoints
- New dashboard pages
- Enhanced memory management
- New authentication flows
- Database migrations

## Success Criteria
- [ ] Both old and new gallery implementations work
- [ ] No build errors
- [ ] Clean commit history
- [ ] Documentation of differences
- [ ] Ability to switch between implementations

## Risks
- **Complex merge conflicts** in shared files
- **Breaking changes** in dependencies
- **Performance impact** of duplicate components
- **User confusion** with multiple interfaces

## Next Steps
1. Start with core infrastructure files
2. Test each integration step
3. Document conflicts and resolutions
4. Create comparison documentation
5. Plan user migration strategy

## References
- Original issue: `docs/issues/open/manual-merge-strategy-gallery-duplication.md`
- Git history analysis: `docs/issues/open/find-pre-merge-state-git-history-complexity.md`
- Pre-merge branch: `552020/icp-396-restore-global-order-only-one-will-survive`
- Main snapshot: `snapshot/icp-396-main-state-2025-01-10`

## Status
- **Created**: 2025-01-10
- **Status**: Ready for implementation
- **Assignee**: TBD
- **Priority**: High
