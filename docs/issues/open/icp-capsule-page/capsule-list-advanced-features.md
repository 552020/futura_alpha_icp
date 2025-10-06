# Capsule List: Advanced Features (Post-MVP)

## Problem Statement

After the MVP capsule list is implemented, users will need advanced features for better capsule management:

1. **Sorting** - Sort capsules by different criteria
2. **Filtering** - Filter capsules by type, status, etc.
3. **Search** - Search capsules by name, description, etc.
4. **Bulk Operations** - Select multiple capsules for bulk actions

## Proposed Solution

Enhance the existing `CapsuleList` component with advanced features for better user experience.

## Features

### Sorting
- **Sort by:** Subject, Role, Web2 Link, Storage, Memories, Galleries, Connections, Space, Lifetime
- **Sort direction:** Ascending, Descending
- **Multi-column sorting:** Sort by multiple criteria
- **Persistent sorting:** Remember user's sorting preferences

### Filtering
- **Capsule type:** Self, Other, All
- **Web2 status:** Connected, ICP Only, All
- **Storage type:** Independent, Shared, All
- **Role:** Owner, Controller, Both, All
- **Date range:** Created, Updated, Expires
- **Storage usage:** Low, Medium, High usage

### Search
- **Global search:** Search across all fields
- **Field-specific search:** Search within specific columns
- **Fuzzy search:** Find similar matches
- **Search history:** Remember recent searches

### Bulk Operations
- **Select all:** Select all visible capsules
- **Select none:** Deselect all capsules
- **Select by filter:** Select all capsules matching current filter
- **Bulk actions:** Delete, Export, Tag, Move

## Implementation Plan

### Phase 1: Sorting
- [ ] **1.1** Add sortable column headers
- [ ] **1.2** Implement single-column sorting
- [ ] **1.3** Add sort indicators (arrows)
- [ ] **1.4** Add persistent sorting state

### Phase 2: Filtering
- [ ] **2.1** Add filter dropdowns
- [ ] **2.2** Implement filter logic
- [ ] **2.3** Add filter chips
- [ ] **2.4** Add clear filters functionality

### Phase 3: Search
- [ ] **3.1** Add search input
- [ ] **3.2** Implement search logic
- [ ] **3.3** Add search suggestions
- [ ] **3.4** Add search history

### Phase 4: Bulk Operations
- [ ] **4.1** Add row selection checkboxes
- [ ] **4.2** Add bulk action toolbar
- [ ] **4.3** Implement bulk operations
- [ ] **4.4** Add confirmation dialogs

## UI Design

### Sorting
```
┌─────────────────────────────────────────────────────────┐
│ Subject ↑    Role    Web2 Link    Storage    Actions    │
├─────────────────────────────────────────────────────────┤
│ You          Owner   Connected    Shared     [View]     │
│ John Doe     Controller ICP Only   Shared     [View]     │
└─────────────────────────────────────────────────────────┘
```

### Filtering
```
┌─────────────────────────────────────────────────────────┐
│ [All Types ▼] [All Status ▼] [All Storage ▼] [Search]   │
├─────────────────────────────────────────────────────────┤
│ [Self] [Connected] [Shared] [Clear Filters]             │
└─────────────────────────────────────────────────────────┘
```

### Bulk Operations
```
┌─────────────────────────────────────────────────────────┐
│ ☑ Select All    [Delete] [Export] [Tag] [Move] (3)     │
├─────────────────────────────────────────────────────────┤
│ ☑ You          Owner   Connected    Shared     [View]  │
│ ☑ John Doe     Controller ICP Only   Shared     [View]  │
│ ☑ Jane Smith   Owner   Connected    Shared     [View]  │
└─────────────────────────────────────────────────────────┘
```

## Technical Considerations

### State Management
- **Sorting state:** Current sort column and direction
- **Filter state:** Active filters and their values
- **Search state:** Search query and results
- **Selection state:** Selected capsule IDs

### Performance
- **Virtual scrolling:** For large capsule lists
- **Debounced search:** Prevent excessive API calls
- **Cached results:** Cache filtered/sorted results
- **Lazy loading:** Load capsules on demand

### Accessibility
- **Keyboard navigation:** Full keyboard support
- **Screen reader support:** Proper ARIA labels
- **Focus management:** Clear focus indicators
- **Color contrast:** Accessible color schemes

## Success Criteria

- [ ] Users can sort capsules by any column
- [ ] Users can filter capsules by multiple criteria
- [ ] Users can search capsules effectively
- [ ] Users can perform bulk operations
- [ ] All features are accessible
- [ ] Performance is maintained with large datasets

## Future Enhancements

- **Advanced filters:** Date ranges, numeric ranges, custom filters
- **Saved views:** Save and restore filter/sort combinations
- **Export options:** Export filtered results to CSV/JSON
- **Real-time updates:** Live updates when capsules change
- **Analytics:** Usage analytics for sorting/filtering patterns

## Files to Modify

- `src/nextjs/src/components/icp/capsule-list.tsx` - Add advanced features
- `src/nextjs/src/components/icp/capsule-list-filters.tsx` - Filter component
- `src/nextjs/src/components/icp/capsule-list-search.tsx` - Search component
- `src/nextjs/src/components/icp/capsule-list-bulk-actions.tsx` - Bulk actions
- `src/nextjs/src/hooks/use-capsule-list.ts` - List management hook

## Dependencies

- **MVP capsule list** must be completed first
- **Backend filtering** may be needed for large datasets
- **Search indexing** may be needed for fast search
- **Bulk operation APIs** may be needed for bulk actions


