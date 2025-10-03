# Internet Identity State Management - Implementation To-Do

## 📋 **Issue Summary**

**Status**: 🔄 **IN PROGRESS** - Phase 1 Complete, Phase 2 Next

**Goal**: Implement the final simplified architecture with no "active" state, no TTL, and clear ownership boundaries between NextAuth and ICP Auth Client.

## 📚 **Related Documents**

### **Architecture Analysis**

- **[Internet Identity State Management Architecture Analysis](internet-identity-state-management-architecture-analysis.md)** - Comprehensive architecture analysis and tech lead's final decision
- **[JWT vs Session Architecture Analysis](jwt-vs-session-architecture-analysis.md)** - Deep dive into JWT vs Session responsibilities and tech lead's definitive answer

### **Key Architecture Decisions**

- ✅ **NO `activeIcPrincipal` in JWT/Session** - Tech lead's definitive answer
- ✅ **8 detailed reasons** why this approach fails
- ✅ **Concrete failure cases** showing the problems
- ✅ **Clear ownership boundaries** for each component
- ✅ **Simplified schema** with only `linkedIcPrincipals: string[]`
- ✅ **Fresh verification at call time** using ICP Auth Client

## 🎯 **Implementation Tasks**

### **Phase 1: Schema Migration**

#### **1.1 Update JWT Interface**

- [x] Remove `activeIcPrincipal` field from JWT interface
- [x] Remove `activeIcPrincipalAssertedAt` field from JWT interface
- [x] Change `linkedIcPrincipal` to `linkedIcPrincipals: string[]` in JWT interface
- [x] Add `linkedIcPrincipals?: string[]` to JWT interface
- [x] Update JWT interface in `src/nextjs/auth.ts`

#### **1.2 Update Session Interface**

- [x] Remove `icpPrincipal` field from SessionUser interface
- [x] Remove `icpPrincipalAssertedAt` field from SessionUser interface
- [x] Change `linkedIcPrincipal` to `linkedIcPrincipals: string[]` in SessionUser interface
- [x] Update SessionUser interface in `src/nextjs/auth.ts`

#### **1.3 Simplify JWT Callback**

- [x] Remove complex TTL logic from JWT callback
- [x] Remove database lookups for `linkedIcPrincipal` in JWT callback
- [x] Add `getLinkedPrincipalsFromDB()` function call on signin
- [x] Implement simplified JWT callback logic per tech lead's specification
- [x] Remove `activeIcPrincipal` handling from JWT callback
- [x] Remove `activeIcPrincipalAssertedAt` handling from JWT callback

#### **1.4 Simplify Session Callback**

- [x] Remove `icpPrincipal` mapping from session callback
- [x] Remove `icpPrincipalAssertedAt` mapping from session callback
- [x] Update `linkedIcPrincipals` mapping in session callback
- [x] Simplify session callback to only map `loginProvider` and `linkedIcPrincipals`

#### **1.5 Type Definitions & Database Schema**

- [ ] **OPTIONAL**: Create `next-auth.d.ts` with proper type augmentation (MVP: inline declarations work)
- [x] **REQUIRED**: Add `loginProvider` type definitions (already done in auth.ts)
- [x] **REQUIRED**: Verify `accounts` table structure matches tech lead's spec (already matches perfectly)
- [x] **REQUIRED**: Add `getLinkedPrincipalsFromDB()` helper function
- [x] **REQUIRED**: Ensure proper Drizzle ORM integration

### **Phase 2: Hook Migration**

#### **2.1 Create New `useIILinks()` Hook**

- [x] Create new file `src/nextjs/src/hooks/use-ii-links.ts`
- [x] Implement thin wrapper over `useSession()`
- [x] Return `status`, `linkedIcPrincipals`, `linkII`, `unlinkII`, `refreshLinks` helpers
- [x] Remove TTL-related state and logic
- [x] Remove `activeIcPrincipal` state and logic
- [x] Remove `isCoAuthActive` state and logic

#### **2.2 Update Components to Use New Hook**

- [ ] Update `LinkedAccounts` component to use `useIILinks()`
- [ ] Update `IICoAuthControls` component to use `useIILinks()`
- [ ] Remove all references to `activeIcPrincipal` in components
- [ ] Remove all references to `isCoAuthActive` in components
- [ ] Remove all TTL-related UI elements
- [ ] Update component interfaces to match new hook

#### **2.3 Remove Old Hook**

- [ ] Delete `src/nextjs/src/hooks/use-ii-coauth.ts`
- [ ] Remove all imports of `useIICoAuth` hook
- [ ] Update component imports to use `useIILinks`

#### **2.4 Ephemeral Identity Management**

- [ ] Create `useICPIdentity()` hook for runtime principal
- [ ] Add avatar component with linked/unlinked badge
- [ ] Implement current auth-client state management
- [ ] Add `useICPIdentity()` for current auth-client state

### **Phase 3: API Implementation**

#### **3.1 Create Link API Endpoint**

- [ ] Create `src/nextjs/src/app/api/auth/ii/link/route.ts`
- [ ] Implement POST handler with `{ principal, proof }` body
- [ ] Integrate with existing challenge/verification system
- [ ] Verify proof using existing `verifyProof()` function
- [ ] Upsert into accounts table with `provider='internet-identity'`
- [ ] Read all linked principals from DB
- [ ] Call `unstable_update({ linkedIcPrincipals })`
- [ ] Add proper error handling and validation

#### **3.2 Create Unlink API Endpoint**

- [ ] Create `src/nextjs/src/app/api/auth/ii/unlink/route.ts`
- [ ] Implement POST handler with `{ principal }` body
- [ ] Delete from accounts table for specified principal
- [ ] Read all linked principals from DB
- [ ] Call `unstable_update({ linkedIcPrincipals })`
- [ ] Add proper error handling and validation

#### **3.3 Create Linked API Endpoint**

- [ ] Create `src/nextjs/src/app/api/auth/ii/linked/route.ts`
- [ ] Implement GET handler
- [ ] Read all linked principals from DB
- [ ] Return current array of linked principals
- [ ] Add proper error handling

#### **3.4 Remove Old API Endpoints**

- [ ] Remove `/api/auth/ii/activate` endpoint (no longer needed)
- [ ] Update any references to old activate endpoint
- [ ] Clean up unused API route files

#### **3.5 Policy & Security Implementation**

- [ ] Add `verifyIIProofOrThrow()` function
- [ ] Implement server-side principal verification
- [ ] Add protected API route validation
- [ ] Ensure proper security boundaries
- [ ] Add server-side principal verification for protected routes

### **Phase 4: Component Updates**

#### **4.1 Update LinkedAccounts Component**

- [ ] Remove `activeIcPrincipal` display logic
- [ ] Remove `isCoAuthActive` state logic
- [ ] Update to show list of linked principals only
- [ ] Add "Link new" button functionality
- [ ] Add "Unlink" button functionality for each principal
- [ ] Update component to use `useIILinks()` hook
- [ ] Remove TTL-related UI elements

#### **4.2 Update IICoAuthControls Component**

- [ ] Remove `activeIcPrincipal` display logic
- [ ] Remove `isCoAuthActive` state logic
- [ ] Remove "Activate" button functionality
- [ ] Remove "Extend Session" button functionality
- [ ] Remove "Disconnect for This Session" button functionality
- [ ] Update to show linked principals list only
- [ ] Add "Link new" button functionality
- [ ] Add "Unlink" button functionality for each principal
- [ ] Update component to use `useIILinks()` hook
- [ ] Remove TTL-related UI elements

#### **4.3 Update ICP Page Component**

- [ ] Remove local authentication state (`isAuthenticated`, `principalId`)
- [ ] Remove `handleLogin()` function
- [ ] Remove `handleLogout()` function
- [ ] Remove `handleGreetSubmit()` function
- [ ] Remove `getAuthenticatedActor()` function
- [ ] Update to use `useIILinks()` hook for linked principals display
- [ ] Update canister calls to use ICP Auth Client directly
- [ ] Remove all references to local authentication state

### **Phase 5: Database Migration**

#### **5.1 Create Migration Script**

- [ ] Create migration to handle existing `linkedIcPrincipal` → `linkedIcPrincipals`
- [ ] Create migration to remove `activeIcPrincipal` and `activeIcPrincipalAssertedAt` fields
- [ ] Add lazy migration logic in JWT callback for existing users
- [ ] Test migration with existing user data

#### **5.2 Update Database Schema**

- [ ] Remove `activeIcPrincipal` field from accounts table if exists
- [ ] Remove `activeIcPrincipalAssertedAt` field from accounts table if exists
- [ ] Ensure `linkedIcPrincipals` array is properly handled in database
- [ ] Update any database queries to use new schema

### **Phase 6: Testing & Validation**

#### **6.1 Unit Tests**

- [ ] Test new `useIILinks()` hook
- [ ] Test link/unlink API endpoints
- [ ] Test JWT callback with new schema
- [ ] Test session callback with new schema
- [ ] Test database migration logic

#### **6.2 Integration Tests**

- [ ] Test complete link flow (challenge → verify → link → display)
- [ ] Test complete unlink flow (unlink → update UI)
- [ ] Test multiple principals linking/unlinking
- [ ] Test canister calls with different linked principals
- [ ] Test session persistence across page reloads

#### **6.3 UI Testing**

- [ ] Test LinkedAccounts component with multiple principals
- [ ] Test IICoAuthControls component with multiple principals
- [ ] Test ICP page with linked principals display
- [ ] Test link/unlink functionality in UI
- [ ] Test error handling in UI

#### **6.4 Edge Case Testing**

- [ ] Test with no linked principals
- [ ] Test with multiple linked principals
- [ ] Test link/unlink with network errors
- [ ] Test session update failures
- [ ] Test database connection failures

#### **6.5 Comprehensive Testing (Tech Lead's Test Checklist)**

- [ ] Test sign-in population of JWT/session
- [ ] Test link/unlink endpoint functionality
- [ ] Test avatar component with linked/unlinked states
- [ ] Test protected API rejection of invalid II proof
- [ ] Test database consistency and session updates
- [ ] Test ephemeral identity hook functionality
- [ ] Test policy enforcement on protected routes

### **Phase 7: Documentation & Cleanup**

#### **7.1 Update Documentation**

- [ ] Update component documentation
- [ ] Update API documentation
- [ ] Update hook documentation
- [ ] Update architecture documentation
- [ ] Remove old documentation references

#### **7.2 Code Cleanup**

- [ ] Remove unused imports
- [ ] Remove unused functions
- [ ] Remove unused types
- [ ] Remove unused API endpoints
- [ ] Clean up console.log statements
- [ ] Update comments and JSDoc

#### **7.3 Performance Optimization**

- [ ] Add caching for `getLinkedPrincipalsFromDB()` if needed
- [ ] Optimize database queries
- [ ] Add error boundaries for components
- [ ] Add loading states for API calls

## 🎯 **Success Criteria**

### **Functional Requirements**

- [ ] Users can link multiple II principals to their account
- [ ] Users can unlink II principals from their account
- [ ] Linked principals are displayed in UI components
- [ ] Canister calls use ICP Auth Client's current identity
- [ ] No "active" state management needed
- [ ] No TTL system needed

### **Technical Requirements**

- [ ] NextAuth stores only linked principals (informational)
- [ ] ICP Auth Client controls which identity is used
- [ ] Clear ownership boundaries between systems
- [ ] No complex state synchronization
- [ ] Simple, maintainable code

### **User Experience Requirements**

- [ ] Clear UI showing linked principals
- [ ] Easy link/unlink functionality
- [ ] No confusion about "active" vs "linked" states
- [ ] User has full control over which II to use

## 📊 **Progress Tracking**

**Overall Progress**: 21% (19/95 tasks completed)

**Phase 1 (Schema Migration)**: ✅ **100%** (18/18 tasks completed)
**Phase 2 (Hook Migration)**: 6% (1/16 tasks)
**Phase 3 (API Implementation)**: 0% (0/21 tasks)
**Phase 4 (Component Updates)**: 0% (0/18 tasks)
**Phase 5 (Database Migration)**: 0% (0/8 tasks)
**Phase 6 (Testing & Validation)**: 0% (0/27 tasks)
**Phase 7 (Documentation & Cleanup)**: 0% (0/12 tasks)

## 🚀 **Next Steps**

1. ✅ **Phase 1 Complete**: JWT and Session interfaces updated
2. **Next: Phase 2**: Create `useIILinks()` hook to replace `useIICoAuth()`
3. **Update components**: Migrate to new architecture
4. **Create APIs**: Implement link/unlink endpoints
5. **Test thoroughly**: Ensure all functionality works
6. **Clean up**: Remove old code and documentation

---

**Priority**: 🔴 **HIGH** - Core architecture change affecting multiple components and systems.

**Estimated Effort**: 5-7 days for complete implementation and testing.

**Dependencies**: Tech lead's final NextAuth callbacks and API handler implementations.

## ✅ **Tech Lead's Final Comprehensive Architecture**

### **What We Already Have Planned** ✅

#### **✅ Schema & Types (Phase 1)**

- ✅ JWT interface with `linkedIcPrincipals: string[]`
- ✅ Session interface with `linkedIcPrincipals: string[]`
- ✅ NextAuth callbacks simplified

#### **✅ Hook Architecture (Phase 2)**

- ✅ `useIILinks()` hook planned
- ✅ Component migration planned
- ✅ Old hook removal planned

#### **✅ API Endpoints (Phase 3)**

- ✅ `/api/auth/ii/link` endpoint planned
- ✅ `/api/auth/ii/unlink` endpoint planned
- ✅ `/api/auth/ii/linked` endpoint planned

### **What We Need to Add** ❌

#### **❌ Missing: Type Definitions**

- [ ] Create `next-auth.d.ts` with proper type augmentation
- [ ] Add `loginProvider` type definitions
- [ ] Ensure proper TypeScript support

#### **❌ Missing: Database Schema Updates**

- [ ] Verify `accounts` table structure matches tech lead's spec
- [ ] Add `getLinkedPrincipalsFromDB()` helper function
- [ ] Ensure proper Drizzle ORM integration

#### **❌ Missing: Ephemeral Identity Hook**

- [ ] Create `useICPIdentity()` hook for runtime principal
- [ ] Add avatar component with linked/unlinked badge
- [ ] Implement `useICPIdentity()` for current auth-client state

#### **❌ Missing: Policy Implementation**

- [ ] Add server-side principal verification
- [ ] Implement `verifyIIProofOrThrow()` function
- [ ] Add protected API route validation

#### **❌ Missing: Test Coverage**

- [ ] Add comprehensive test checklist
- [ ] Test sign-in population of JWT/session
- [ ] Test link/unlink endpoint functionality
- [ ] Test avatar component with linked/unlinked states
- [ ] Test protected API rejection of invalid II proof

### **✅ Integration Complete**

All missing pieces from the tech lead's final architecture have been properly integrated into our existing plan structure:

#### **✅ Phase 1.5: Type Definitions & Database Schema** (Added to Phase 1)

- ✅ Integrated into Phase 1 as section 1.5
- ✅ Includes `next-auth.d.ts`, `loginProvider` types, database schema verification

#### **✅ Phase 2.4: Ephemeral Identity Management** (Added to Phase 2)

- ✅ Integrated into Phase 2 as section 2.4
- ✅ Includes `useICPIdentity()` hook, avatar component, auth-client state management

#### **✅ Phase 3.5: Policy & Security Implementation** (Added to Phase 3)

- ✅ Integrated into Phase 3 as section 3.5
- ✅ Includes `verifyIIProofOrThrow()`, server-side verification, protected API validation

#### **✅ Phase 6.5: Comprehensive Testing** (Added to Phase 6)

- ✅ Integrated into Phase 6 as section 6.5
- ✅ Includes tech lead's complete test checklist for all functionality

### **📊 Updated Task Counts:**

- **Total Tasks**: 95 (increased from 87)
- **Phase 2**: 16 tasks (increased from 12)
- **Phase 3**: 21 tasks (increased from 16)
- **Phase 6**: 27 tasks (increased from 20)

### **🎯 No Tasks Will Be Skipped**

All missing pieces from the tech lead's final architecture are now properly linked and integrated into our existing plan structure, ensuring nothing gets skipped during implementation.
