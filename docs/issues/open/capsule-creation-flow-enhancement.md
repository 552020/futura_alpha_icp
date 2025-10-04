# Capsule Creation Flow Enhancement

## Problem Statement

The current capsule creation flow needs enhancement to provide a better user experience:

1. **Self-capsule auto-creation**: Users should automatically get their self-capsule on signin/signup
2. **Additional capsule creation**: Users need a way to create capsules for other subjects (deceased people, minors, organizations) with proper subject specification

## Current State

- ✅ Basic capsule creation works
- ✅ Self-capsule creation works (idempotent)
- ❌ No auto-creation on signin/signup
- ❌ No UI for specifying subject when creating additional capsules
- ❌ No modal for capsule creation data entry

## Proposed Solution

### 1. Auto-Creation on Signin/Signup

**Implementation:**

- Modify authentication flow to automatically create self-capsule
- Add `ensureSelfCapsule()` function in service layer
- Call during user authentication process

**Technical Details:**

```typescript
// In authentication hook or service
export async function ensureSelfCapsule(getActor: () => Promise<BackendActor>): Promise<Capsule> {
  try {
    // Try to get existing self-capsule
    const existing = await getCapsuleFull(getActor, clearActor);
    if (existing) return existing;

    // Create self-capsule if none exists
    return await createCapsule(null, getActor, clearActor);
  } catch (error) {
    // Handle errors gracefully
    throw error;
  }
}
```

**Integration Points:**

- `useAuthenticatedActor` hook
- Auth.js signin callback
- Component mount in `CapsuleInfo`

### 2. Enhanced Create Capsule Button

**UI Changes:**

- Replace simple "Create Capsule" button with "Create New Capsule" button
- Add modal for capsule creation with subject specification
- Include form fields for:
  - Subject type (Principal, Opaque)
  - Subject value (for Opaque type)
  - Capsule purpose/description (optional)

**Modal Design:**

```typescript
interface CreateCapsuleFormData {
  subjectType: "self" | "principal" | "opaque";
  subjectValue?: string; // For opaque type
  description?: string; // Optional
}
```

**Form Fields:**

1. **Subject Type Selection:**

   - Radio buttons: "Self", "Principal", "Other"
   - Show different inputs based on selection

2. **Subject Value Input:**

   - For "Principal": Principal input field
   - For "Other": Text input for opaque value
   - For "Self": No additional input needed

3. **Optional Description:**
   - Text area for capsule purpose
   - Help text: "What is this capsule for? (e.g., 'My grandmother's memories')"

### 3. Backend Integration

**Service Layer Updates:**

```typescript
export async function createCapsuleWithSubject(
  formData: CreateCapsuleFormData,
  getActor: () => Promise<BackendActor>,
  clearActor: () => void
): Promise<Capsule> {
  let subject: PersonRef | null = null;

  switch (formData.subjectType) {
    case "self":
      subject = null; // Will create self-capsule
      break;
    case "principal":
      subject = { Principal: formData.subjectValue };
      break;
    case "opaque":
      subject = { Opaque: formData.subjectValue };
      break;
  }

  return await createCapsule(subject, getActor, clearActor);
}
```

## Implementation Plan

### Phase 1: Auto-Creation on Signin

- [ ] **1.1** Add `ensureSelfCapsule()` function to service layer
- [ ] **1.2** Integrate with authentication flow
- [ ] **1.3** Update `CapsuleInfo` component to handle auto-creation
- [ ] **1.4** Test auto-creation flow
- [ ] **1.5** Handle error cases (creation fails)

### Phase 2: Enhanced Create Button

- [ ] **2.1** Create `CreateCapsuleModal` component
- [ ] **2.2** Add form validation for subject input
- [ ] **2.3** Implement subject type selection logic
- [ ] **2.4** Add form submission handling
- [ ] **2.5** Update main component to use modal
- [ ] **2.6** Test modal functionality

### Phase 3: Backend Integration

- [ ] **3.1** Update service layer with `createCapsuleWithSubject()`
- [ ] **3.2** Add proper error handling for different subject types
- [ ] **3.3** Test with various subject combinations
- [ ] **3.4** Add success/error feedback

### Phase 4: UI/UX Polish

- [ ] **4.1** Add loading states during creation
- [ ] **4.2** Improve error messages for different failure cases
- [ ] **4.3** Add success animations/feedback
- [ ] **4.4** Test complete user flow

## Technical Considerations

### Error Handling

- **Self-capsule already exists**: Handle gracefully (idempotent)
- **Invalid subject format**: Show validation errors
- **Backend creation fails**: Show specific error messages
- **Network issues**: Retry logic with exponential backoff

### User Experience

- **Auto-creation**: Silent for self-capsule, show loading indicator
- **Manual creation**: Clear form, validation, success feedback
- **Error states**: Specific messages for different failure types

### Performance

- **Auto-creation**: Should not block signin flow
- **Modal**: Lazy load to avoid bundle bloat
- **Form validation**: Client-side validation before API calls

## Success Criteria

- [ ] ✅ Users automatically get self-capsule on first signin
- [ ] ✅ Users can create additional capsules with proper subject specification
- [ ] ✅ Clear error handling for all failure cases
- [ ] ✅ Smooth user experience with appropriate loading states
- [ ] ✅ No regressions in existing functionality

## Future Enhancements

- **Capsule templates**: Pre-defined templates for common use cases
- **Bulk creation**: Create multiple capsules at once
- **Import/Export**: Import capsule data from external sources
- **Advanced permissions**: Set up complex ownership/control relationships

## Files to Modify

- `src/nextjs/src/services/capsule.ts` - Add `ensureSelfCapsule()` and `createCapsuleWithSubject()`
- `src/nextjs/src/components/icp/capsule-info.tsx` - Update to use auto-creation
- `src/nextjs/src/components/icp/create-capsule-modal.tsx` - New modal component
- `src/nextjs/src/hooks/use-authenticated-actor.ts` - Integrate auto-creation
- `src/nextjs/src/types/capsule.ts` - Add form data types

## Dependencies

- Modal component library (existing)
- Form validation library (existing)
- Toast notifications (existing)
- Authentication hooks (existing)
