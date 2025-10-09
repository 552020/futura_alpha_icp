# Database Switching Comprehensive Testing

**Priority**: High  
**Type**: Testing & Validation  
**Status**: Ready for Testing  
**Created**: 2025-01-16  
**Related**: ICP-413 Wire ICP Memory Upload Frontend-Backend

## 🎯 **Objective**

Comprehensive testing of the database switching functionality to verify that users can seamlessly switch between viewing memories stored in ICP (Internet Computer Protocol) and Neon databases, with proper upload and deletion functionality across both storage systems.

## 📋 **Testing Overview**

The implementation is complete and ready for testing. This document outlines comprehensive test scenarios to validate:

1. **Database Switching**: ICP/Neon toggle functionality in dashboard
2. **Upload Flow**: File uploads to selected storage systems
3. **Settings Integration**: Hosting preference configuration
4. **Clear All**: Memory deletion across both databases
5. **Error Handling**: Graceful fallbacks and error recovery

## 🧪 **Test Scenarios**

### **Scenario 1: Web2 Only Configuration**

**Setup**: User has only Web2 stack enabled (Neon database + S3 blob storage)

**Test Steps**:

1. Go to Profile Settings → Hosting Preferences
2. Verify only "Web2" checkbox is enabled (Backend: Vercel, Database: Neon, Blob: S3)
3. Go to Dashboard
4. Verify database toggle is **hidden** (only one database available)
5. Upload a test file/folder
6. Verify file appears in dashboard
7. Verify file is stored in Neon database
8. Test "Clear All" functionality
9. Verify all memories are deleted from Neon

**Expected Results**:

- ✅ Database toggle not visible (single database)
- ✅ Uploads go to Neon database
- ✅ Memories display correctly
- ✅ Clear All works for Neon memories

### **Scenario 2: Web3 Only Configuration**

**Setup**: User has only Web3 stack enabled (ICP database + ICP blob storage)

**Test Steps**:

1. Go to Profile Settings → Hosting Preferences
2. Enable only "Web3" checkbox (Backend: ICP, Database: ICP, Blob: ICP)
3. Go to Dashboard
4. Verify database toggle is **hidden** (only one database available)
5. Upload a test file/folder
6. Verify file appears in dashboard
7. Verify file is stored in ICP canister
8. Test "Clear All" functionality
9. Verify all memories are deleted from ICP

**Expected Results**:

- ✅ Database toggle not visible (single database)
- ✅ Uploads go to ICP canister
- ✅ Memories display correctly with ICP data transformation
- ✅ Clear All works for ICP memories

### **Scenario 3: Dual Stack Configuration (Primary Test)**

**Setup**: User has both Web2 and Web3 stacks enabled

**Test Steps**:

1. Go to Profile Settings → Hosting Preferences
2. Enable both "Web2" and "Web3" checkboxes
3. Go to Dashboard
4. Verify database toggle is **visible** with both options
5. Test database switching:
   - Switch to "Neon" view
   - Upload a test file
   - Switch to "ICP" view
   - Upload another test file
   - Switch between views to verify memories appear in correct database
6. Test "Clear All" functionality
7. Verify memories are deleted from both databases

**Expected Results**:

- ✅ Database toggle visible with both options
- ✅ Can switch between ICP and Neon views
- ✅ Uploads go to selected database
- ✅ Memories display correctly in each view
- ✅ Clear All works for both databases

### **Scenario 4: Database Switching Functionality**

**Setup**: User with dual stack configuration

**Test Steps**:

1. Ensure both Web2 and Web3 are enabled
2. Go to Dashboard
3. Verify toggle shows "Neon" by default
4. Upload 2-3 test files while in "Neon" view
5. Switch toggle to "ICP"
6. Verify dashboard shows ICP memories (may be empty initially)
7. Upload 2-3 test files while in "ICP" view
8. Switch back to "Neon" view
9. Verify Neon memories are still there
10. Switch to "ICP" view
11. Verify ICP memories are still there
12. Test switching multiple times rapidly

**Expected Results**:

- ✅ Toggle switches between views instantly
- ✅ Each view shows correct memories
- ✅ Uploads go to currently selected database
- ✅ No data loss when switching
- ✅ Smooth user experience

### **Scenario 5: Error Handling & Fallbacks**

**Setup**: Various error conditions

**Test Steps**:

1. **ICP Connection Failure**:
   - Disconnect from Internet Identity
   - Try to switch to ICP view
   - Verify graceful fallback to Neon
2. **Neon API Failure**:
   - Simulate network issues
   - Try to fetch Neon memories
   - Verify error handling
3. **Mixed Authentication**:
   - Have both NextAuth and Internet Identity
   - Test switching between databases
   - Verify proper authentication for each

**Expected Results**:

- ✅ Graceful fallback when ICP unavailable
- ✅ Clear error messages for users
- ✅ No application crashes
- ✅ Proper authentication handling

### **Scenario 6: Upload Flow Testing**

**Setup**: Dual stack configuration

**Test Steps**:

1. **Neon Upload**:
   - Set database toggle to "Neon"
   - Upload a file/folder
   - Verify file appears in Neon view
   - Verify file does NOT appear in ICP view
2. **ICP Upload**:
   - Set database toggle to "ICP"
   - Upload a file/folder
   - Verify file appears in ICP view
   - Verify file does NOT appear in Neon view
3. **Upload Progress**:
   - Upload large files
   - Verify progress indicators work
   - Verify success/error messages

**Expected Results**:

- ✅ Uploads go to correct database
- ✅ Files appear only in selected database view
- ✅ Progress indicators work correctly
- ✅ Success/error feedback is clear

### **Scenario 7: Clear All Functionality**

**Setup**: Dual stack with memories in both databases

**Test Steps**:

1. Upload files to both Neon and ICP
2. Verify memories exist in both views
3. Click "Clear All" button
4. Verify confirmation dialog appears
5. Confirm deletion
6. Verify all memories are deleted from both databases
7. Test with only one database having memories

**Expected Results**:

- ✅ Clear All deletes from both databases
- ✅ Confirmation dialog prevents accidental deletion
- ✅ Works with single database memories
- ✅ Works with dual database memories

## 🔧 **Technical Validation**

### **Data Format Compatibility**

**Test**: Verify ICP memories display correctly in dashboard format

**Validation Points**:

- ✅ Memory titles and descriptions display
- ✅ Timestamps are correctly formatted
- ✅ Memory types (image, video, etc.) are correct
- ✅ Folder grouping works with ICP memories
- ✅ Asset URLs are accessible

### **Performance Testing**

**Test**: Verify database switching is responsive

**Validation Points**:

- ✅ Switching between views is fast (< 2 seconds)
- ✅ Memory loading is responsive
- ✅ No UI blocking during switches
- ✅ React Query caching works correctly

### **Authentication Integration**

**Test**: Verify proper authentication for each database

**Validation Points**:

- ✅ Neon access uses NextAuth session
- ✅ ICP access uses Internet Identity
- ✅ Dual authentication works correctly
- ✅ Unauthenticated access is handled gracefully

## 📊 **Success Criteria**

### **Functional Requirements**

- [ ] Database toggle appears only when multiple databases are available
- [ ] Users can switch between ICP and Neon views seamlessly
- [ ] Uploads go to the currently selected database
- [ ] Memories display correctly in both views
- [ ] Clear All works for both databases
- [ ] Error handling provides graceful fallbacks

### **User Experience Requirements**

- [ ] Toggle switching is intuitive and responsive
- [ ] Loading states provide clear feedback
- [ ] Error messages are helpful and actionable
- [ ] Upload progress is visible and accurate
- [ ] Settings changes take effect immediately

### **Technical Requirements**

- [ ] No data loss during database switching
- [ ] Proper authentication for each database
- [ ] React Query caching works correctly
- [ ] Error handling is comprehensive
- [ ] Performance is acceptable for typical use cases

## 🐛 **Known Issues to Watch For**

1. **ICP Connection Issues**: Internet Identity authentication failures
2. **Data Transformation**: ICP memory format compatibility
3. **Upload Routing**: Files going to wrong database
4. **Clear All**: Partial deletion failures
5. **Toggle State**: UI state not syncing with actual data source

## 📝 **Test Data Requirements**

### **Test Files Needed**

- **Images**: JPG, PNG, WebP (various sizes)
- **Videos**: MP4, WebM (small and large)
- **Documents**: PDF, TXT, MD
- **Mixed Folders**: Combination of file types

### **Test Accounts Needed**

- **Web2 Only**: User with only Neon database access
- **Web3 Only**: User with only ICP database access
- **Dual Access**: User with both database access
- **Admin**: User for testing edge cases

## 🎯 **Testing Priority**

### **High Priority (Must Pass)**

1. Database switching functionality
2. Upload flow to correct database
3. Clear All across both databases
4. Basic error handling

### **Medium Priority (Should Pass)**

1. Performance and responsiveness
2. Advanced error scenarios
3. Mixed authentication
4. Data format compatibility

### **Low Priority (Nice to Have)**

1. Edge case handling
2. Advanced UI interactions
3. Performance optimization
4. Detailed error messages

## 📋 **Testing Checklist**

### **Pre-Testing Setup**

- [ ] Ensure both ICP and Neon databases are accessible
- [ ] Verify Internet Identity authentication works
- [ ] Confirm NextAuth session is valid
- [ ] Test with clean database state
- [ ] Prepare test files of various types

### **Core Functionality Testing**

- [ ] Database toggle visibility (single vs dual stack)
- [ ] Database switching between ICP and Neon
- [ ] File uploads to selected database
- [ ] Memory display in both views
- [ ] Clear All functionality
- [ ] Error handling and fallbacks

### **Integration Testing**

- [ ] Settings changes affect dashboard behavior
- [ ] Upload preferences work correctly
- [ ] Authentication flows work properly
- [ ] React Query caching functions correctly
- [ ] UI state management is consistent

### **Post-Testing Cleanup**

- [ ] Clear test data from both databases
- [ ] Reset user preferences to defaults
- [ ] Document any issues found
- [ ] Update test results and status

## 🔗 **Related Issues**

- [Database Switching Implementation](./dashboard-memory-display-flow-analysis.md)
- [Clear All ICP Integration](./clear-all-icp-integration-analysis.md)
- [File Upload Errors](./file-upload-errors-issue.md)
- [Hosting Preferences Logic](./hosting-preferences-toggle-logic-fix.md)

---

**Last Updated**: 2025-01-16  
**Status**: Ready for Testing  
**Priority**: High - Core functionality validation
