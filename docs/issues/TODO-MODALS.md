# Modal Components TODO

This document lists all modal/dialog components currently used in the codebase and their refactoring status.

## 🎯 **Standard Modal System**

### ✅ **Completed**

- **ConfirmationModal** (`src/components/modals/confirmation-modal.tsx`)
  - Generic reusable confirmation dialog
  - Supports loading states, variants (default/destructive)
  - Used as base for specific modals

- **DeleteAccountModal** (`src/components/modals/delete-account-modal.tsx`)
  - ✅ **Refactored** to use ConfirmationModal
  - Handles account deletion with API call
  - Includes sign out and redirect logic

## 📋 **Modals to Refactor**

### 🔄 **High Priority - Core Functionality**

1. **OnboardModal** (`src/components/onboarding/onboard-modal.tsx`)
   - **Purpose**: Multi-step onboarding flow
   - **Status**: ❌ Custom implementation
   - **Refactor**: Extract to use standard modal system
   - **Complexity**: High (multi-step, form handling)

2. **CreateGalleryModal** (`src/components/galleries/create-gallery-modal.tsx`)
   - **Purpose**: Create gallery from folder
   - **Status**: ❌ Custom implementation
   - **Refactor**: Could use ConfirmationModal for confirmation step
   - **Complexity**: Medium (form with folder selection)

3. **CreateCapsuleModal** (`src/components/icp/create-capsule-modal.tsx`)
   - **Purpose**: Create ICP capsule
   - **Status**: ❌ Custom implementation
   - **Refactor**: Extract to standard modal
   - **Complexity**: Medium (form with validation)

### 🔄 **Medium Priority - User Experience**

4. **GalleryImageModal** (`src/components/galleries/gallery-image-modal.tsx`)
   - **Purpose**: Image viewer in gallery
   - **Status**: ❌ Custom implementation
   - **Refactor**: Could use standard modal base
   - **Complexity**: Medium (image navigation, controls)

5. **ForeverStorageProgressModal** (`src/components/galleries/forever-storage-progress-modal.tsx`)
   - **Purpose**: Progress tracking for storage operations
   - **Status**: ❌ Custom implementation
   - **Refactor**: Extract progress modal component
   - **Complexity**: High (progress tracking, real-time updates)

6. **SendSelectionModal** (`src/components/galleries/send-selection-modal.tsx`)
   - **Purpose**: Send gallery selection
   - **Status**: ❌ Custom implementation
   - **Refactor**: Could use ConfirmationModal
   - **Complexity**: Low (confirmation + form)

### 🔄 **Low Priority - Utility**

7. **ShareDialog** (`src/components/memory/share-dialog.tsx`)
   - **Purpose**: Share memory dialog
   - **Status**: ❌ Custom implementation
   - **Refactor**: Extract to standard modal
   - **Complexity**: Low (form with share options)

## 🏗️ **Proposed Modal Architecture**

### **Base Components**

```
src/components/modals/
├── index.ts                           # Export all modals
├── confirmation-modal.tsx            # ✅ Generic confirmation
├── form-modal.tsx                    # 🔄 Generic form modal
├── progress-modal.tsx                # 🔄 Progress tracking modal
└── image-viewer-modal.tsx            # 🔄 Image viewer modal
```

### **Specific Modals**

```
src/components/modals/
├── delete-account-modal.tsx          # ✅ Uses ConfirmationModal
├── delete-memory-modal.tsx           # 🔄 Uses ConfirmationModal
├── delete-folder-modal.tsx           # 🔄 Uses ConfirmationModal
├── create-gallery-modal.tsx          # 🔄 Uses FormModal
├── create-capsule-modal.tsx          # 🔄 Uses FormModal
├── onboard-modal.tsx                # 🔄 Uses FormModal (multi-step)
├── gallery-image-modal.tsx          # 🔄 Uses ImageViewerModal
└── storage-progress-modal.tsx        # 🔄 Uses ProgressModal
```

## 📝 **Refactoring Plan**

### **Phase 1: Base Components**

- [ ] Create `FormModal` component
- [ ] Create `ProgressModal` component
- [ ] Create `ImageViewerModal` component

### **Phase 2: Simple Confirmations**

- [ ] Refactor `ShareDialog` → `ShareModal`
- [ ] Create `DeleteMemoryModal`
- [ ] Create `DeleteFolderModal`
- [ ] Create `DeleteGalleryModal`

### **Phase 3: Form Modals**

- [ ] Refactor `CreateGalleryModal`
- [ ] Refactor `CreateCapsuleModal`
- [ ] Refactor `SendSelectionModal`

### **Phase 4: Complex Modals**

- [ ] Refactor `OnboardModal` (multi-step)
- [ ] Refactor `GalleryImageModal`
- [ ] Refactor `ForeverStorageProgressModal`

## 🎯 **Benefits of Standardization**

### **Consistency**

- Same look and feel across all modals
- Consistent loading states and error handling
- Unified accessibility features

### **Maintainability**

- Single source of truth for modal behavior
- Easy to update styling globally
- Reduced code duplication

### **Developer Experience**

- Easy to create new modals
- Type-safe props and configurations
- Clear separation of concerns

## 🔧 **Implementation Notes**

### **ConfirmationModal Usage**

```tsx
<ConfirmationModal
  isOpen={isOpen}
  onClose={onClose}
  onConfirm={handleDelete}
  title="Delete Memory"
  description="Are you sure you want to delete this memory?"
  variant="destructive"
  confirmText="Delete"
  loadingText="Deleting..."
/>
```

### **FormModal Usage** (Proposed)

```tsx
<FormModal
  isOpen={isOpen}
  onClose={onClose}
  onSubmit={handleSubmit}
  title="Create Gallery"
  description="Create a new gallery from your memories"
  form={form}
  fields={formFields}
  submitText="Create Gallery"
  loadingText="Creating..."
/>
```

### **ProgressModal Usage** (Proposed)

```tsx
<ProgressModal
  isOpen={isOpen}
  onClose={onClose}
  title="Uploading Memories"
  description="Please wait while we upload your memories"
  progress={uploadProgress}
  status="uploading"
  onCancel={handleCancel}
/>
```

## 📊 **Current Status**

- **Total Modals Found**: 8
- **Refactored**: 1 (DeleteAccountModal)
- **Pending Refactor**: 7
- **Base Components**: 1 (ConfirmationModal)
- **Proposed Base Components**: 3 (FormModal, ProgressModal, ImageViewerModal)

---

**Last Updated**: $(date)
**Next Review**: After Phase 1 completion
