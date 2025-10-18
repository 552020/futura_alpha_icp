# 🐛 Onboarding Flow Debug Issue

**Date:** 2024-12-19  
**Status:** In Progress  
**Priority:** High  
**Labels:** `onboarding`, `debug`, `user-flow`, `frontend`

## 📋 **Summary**

Debug the onboarding flow starting from the home page button that navigates users to the items-upload page at `http://localhost:3000/en/onboarding/items-upload`. The flow needs to be analyzed for potential issues, user experience problems, and technical debugging points.

## 🔍 **Current Onboarding Flow**

### **Entry Point: Home Page Button**

- **Location**: Home page (`/en/`)
- **Component**: `HeroDemo` component (when `NEXT_PUBLIC_HERO !== 'vault'`)
- **Button**: Arrow button in bottom-right corner
- **Link**: `/${lang}/onboarding/items-upload`
- **Code Reference**: `src/components/marketing/hero-demo.tsx:218-225`

### **Flow Steps**

1. **Home Page** → User clicks arrow button
2. **Items Upload Page** → `/[lang]/onboarding/items-upload`
3. **Upload Success** → Triggers `OnboardModal`
4. **Modal Completion** → Redirects to `/[lang]/onboarding/profile`

## 🎯 **Debug Points to Investigate**

### **1. Home Page Entry**

- [ ] **Button Visibility**: Is the arrow button properly visible and accessible?
- [ ] **Link Generation**: Does the language parameter get passed correctly?
- [ ] **Authentication Check**: Does the home page properly redirect authenticated users to dashboard?
- [ ] **Segment Handling**: Is the segment cookie properly set and used?

### **2. Items Upload Page**

- [ ] **Page Loading**: Does the page load without errors?
- [ ] **Dictionary Loading**: Are translations properly loaded for the component?
- [ ] **Component Rendering**: Do both upload button variants render correctly?
- [ ] **Configuration Flags**: Are `DOUBLE_BUTTON`, `WITH_SUBTITLE` flags working?

### **3. Upload Process**

- [ ] **File Selection**: Does the file input work correctly?
- [ ] **Upload Success**: Does `handleUploadSuccess` trigger properly?
- [ ] **Modal Display**: Does the `OnboardModal` appear after successful upload?
- [ ] **Context Updates**: Are onboarding context updates working?

### **4. Modal Flow**

- [ ] **Modal Opening**: Does the modal open with correct content?
- [ ] **Step Navigation**: Do users progress through modal steps correctly?
- [ ] **Completion**: Does `handleOnboardingComplete` work properly?
- [ ] **Redirect**: Does the redirect to profile page work?

## 🔧 **Technical Components to Debug**

### **Key Files**

- `src/app/[lang]/page.tsx` - Home page logic
- `src/components/marketing/hero-demo.tsx` - Entry button
- `src/app/[lang]/onboarding/items-upload/page.tsx` - Upload page
- `src/app/[lang]/onboarding/items-upload/items-upload-client.tsx` - Upload logic
- `src/components/onboarding/onboard-modal.tsx` - Modal component
- `src/contexts/onboarding-context.tsx` - State management
- `src/hooks/use-file-upload.ts` - Upload functionality

### **Configuration Flags**

```typescript
// In items-upload-client.tsx
const DOUBLE_BUTTON = false; // Show single or double upload buttons
const WITH_SUBTITLE = true; // Show subtitle text
const EXPERIMENT = false; // Use experiment version
```

### **Context Dependencies**

- `useOnboarding()` - Onboarding state management
- `useInterface()` - Interface mode switching
- `useSession()` - Authentication state
- `useHostingPreferences()` - Upload preferences

## 🐛 **Known Issues to Check**

### **1. Authentication Flow**

- **Issue**: Authenticated users should be redirected to dashboard from home
- **Check**: Does `auth()` work correctly in home page?
- **Debug**: Console logs for session state

### **2. Dictionary Loading**

- **Issue**: Missing translations might cause errors
- **Check**: Are all required dictionary keys present?
- **Debug**: Error handling in `getDictionary()`

### **3. Upload Context**

- **Issue**: Onboarding context might not update properly
- **Check**: Does `updateUserData()` and `addOnboardingFile()` work?
- **Debug**: Context state changes in React DevTools

### **4. Modal State**

- [ ] **Issue**: Modal might not open after upload
- **Check**: Does `setShowOnboardModal(true)` trigger?
- **Debug**: State changes and modal visibility

## 🧪 **Debug Commands**

### **Development Server**

```bash
# Start development server
npm run dev

# Check specific URL
curl http://localhost:3000/en/onboarding/items-upload
```

### **Browser Debugging**

```javascript
// Console commands to check state
console.log("Onboarding context:", window.__REACT_DEVTOOLS_GLOBAL_HOOK__);
console.log("Session:", sessionStorage.getItem("session"));
console.log("Local storage:", localStorage);
```

### **Network Debugging**

- Check for failed API calls
- Verify file upload endpoints
- Monitor authentication requests

## 📊 **Test Scenarios**

### **Scenario 1: New User Flow**

1. Visit `http://localhost:3000/en/`
2. Click arrow button
3. Verify redirect to items-upload page
4. Upload a file
5. Verify modal opens
6. Complete onboarding steps
7. Verify redirect to profile page

### **Scenario 2: Authenticated User**

1. Login to the app
2. Visit `http://localhost:3000/en/`
3. Verify redirect to dashboard (not items-upload)

### **Scenario 3: Upload Failure**

1. Navigate to items-upload page
2. Try to upload invalid file
3. Verify error handling
4. Check error messages

## 🎯 **Success Criteria**

- [ ] Home page button works correctly
- [ ] Items upload page loads without errors
- [ ] File upload process works
- [ ] Modal appears after successful upload
- [ ] Onboarding steps complete successfully
- [ ] Final redirect to profile page works
- [ ] No console errors during the flow
- [ ] Proper error handling for edge cases

## 📝 **Next Steps**

1. **Run the debug scenarios** above
2. **Document any issues** found during testing
3. **Check browser console** for errors
4. **Verify all API calls** are working
5. **Test with different file types** and sizes
6. **Verify mobile responsiveness** of the flow

## 🔗 **Related Issues**

- Authentication flow debugging
- File upload system debugging
- Modal component debugging
- Context state management debugging
