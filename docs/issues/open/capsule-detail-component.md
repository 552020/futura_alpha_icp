# Capsule Detail Component

## Problem Statement

Users need a detailed view of individual capsules to:

- View comprehensive capsule information
- Edit capsule details
- Delete capsules
- Navigate between capsules
- Access capsule-specific actions

## Proposed Solution

Create a `CapsuleDetail` component that provides a comprehensive view of a single capsule with editing capabilities.

## Backend Types Reference

See `docs/backend-api-documentation.md` for complete backend API details.

## Component Structure

### Props Interface

```typescript
interface CapsuleDetailProps {
  capsuleId: string;
  onClose: () => void;
  onEdit: (capsule: Capsule) => void;
  onDelete: (capsuleId: string) => void;
}
```

### State Management

```typescript
interface CapsuleDetailState {
  capsule: Capsule | null;
  isLoading: boolean;
  error: CapsuleError | null;
  isEditing: boolean;
  editData: Partial<CapsuleUpdateData>;
}
```

## UI Design

### Layout Structure

```
┌─────────────────────────────────────────────────────────┐
│ [← Back] Capsule Details                    [Edit] [×] │
├─────────────────────────────────────────────────────────┤
│ Subject: You (Self-capsule)                             │
│ Role: Owner & Controller                               │
│ Web2 Link: Connected                                   │
│ Storage: Shared                                        │
├─────────────────────────────────────────────────────────┤
│ Memories: 15                    [View All] [Add New]    │
│ Galleries: 3                   [View All] [Create]     │
│ Connections: 8                  [View All] [Connect]   │
├─────────────────────────────────────────────────────────┤
│ Created: 2 months ago                                  │
│ Updated: 1 week ago                                    │
│ Last Activity: 3 days ago                               │
└─────────────────────────────────────────────────────────┘
```

### Edit Mode

```
┌─────────────────────────────────────────────────────────┐
│ [← Back] Edit Capsule                       [Save] [×] │
├─────────────────────────────────────────────────────────┤
│ Subject: [You (Self-capsule)]                           │
│ Description: [Text area for capsule description]       │
│ Tags: [Tag input for categorization]                   │
├─────────────────────────────────────────────────────────┤
│ [Cancel] [Save Changes]                                │
└─────────────────────────────────────────────────────────┘
```

## Features

### View Mode

- **Capsule Information**: Display all capsule details
- **Statistics**: Show memory, gallery, and connection counts
- **Activity Timeline**: Recent capsule activity
- **Quick Actions**: Access to common operations

### Edit Mode

- **Form Fields**: Editable capsule properties
- **Validation**: Real-time form validation
- **Save/Cancel**: Clear action buttons
- **Auto-save**: Optional auto-save functionality

### Navigation

- **Back Button**: Return to capsule list
- **Close Button**: Close detail view
- **Previous/Next**: Navigate between capsules

## Implementation Plan

### Phase 1: Basic Detail View

- [ ] **1.1** Create `CapsuleDetail` component structure
- [ ] **1.2** Implement capsule information display
- [ ] **1.3** Add loading and error states
- [ ] **1.4** Add navigation controls

### Phase 2: Edit Functionality

- [ ] **2.1** Add edit mode toggle
- [ ] **2.2** Implement edit form
- [ ] **2.3** Add form validation
- [ ] **2.4** Add save/cancel functionality

### Phase 3: Enhanced Features

- [ ] **3.1** Add activity timeline
- [ ] **3.2** Add quick actions
- [ ] **3.3** Add navigation between capsules
- [ ] **3.4** Add delete confirmation

### Phase 4: Integration

- [ ] **4.1** Integrate with `CapsuleList` component
- [ ] **4.2** Add routing for direct access
- [ ] **4.3** Test complete user flow
- [ ] **4.4** Add responsive design

## Technical Considerations

### Data Fetching

- Use `getCapsuleFull()` for complete capsule data
- Handle loading states and errors
- Implement refresh functionality

### State Management

- Manage edit state locally
- Sync with parent component
- Handle concurrent edits

### Performance

- Lazy load capsule data
- Optimize re-renders
- Cache capsule information

## Success Criteria

- [ ] Users can view detailed capsule information
- [ ] Users can edit capsule details
- [ ] Users can navigate between capsules
- [ ] Users can delete capsules
- [ ] Component is responsive and accessible
- [ ] Integration with capsule list works seamlessly

## Future Enhancements

- **Bulk Operations**: Select multiple capsules for bulk actions
- **Advanced Editing**: Rich text editing for descriptions
- **Activity Feed**: Real-time activity updates
- **Export/Import**: Capsule data export functionality
- **Sharing**: Share capsule information with others

## Files to Create

- `src/nextjs/src/components/icp/capsule-detail.tsx`
- `src/nextjs/src/components/icp/capsule-detail-edit.tsx`
- `src/nextjs/src/components/icp/capsule-detail-actions.tsx`

## Dependencies

- `@/types/capsule` - Capsule types and interfaces
- `@/services/capsule` - Capsule service functions
- `@/components/ui/*` - shadcn/ui components
- `@/lib/icp-error-handling` - Error handling utilities
