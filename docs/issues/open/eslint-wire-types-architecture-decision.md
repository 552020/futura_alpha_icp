# ESLint Wire Types Architecture Decision

## 📋 **Issue Summary**

**Status**: 🔴 **BLOCKING** - Need senior guidance on architectural approach  
**Priority**: High  
**Type**: Architecture Decision

## 🎯 **Problem Statement**

We have a **domain-driven architecture** with ESLint rules that restrict wire types (Candid-generated types) to only be used in `lib/` directory, but ICP implementation files legitimately need direct access to backend types.

## 🔧 **Current Situation**

### **ESLint Rule:**

```javascript
'no-restricted-imports': [
  'error',
  {
    patterns: [
      {
        group: ['@/ic/declarations/backend/backend.did'],
        message: 'Wire types only allowed in lib/ directory. Use domain types from @/types/upload instead.',
      },
    ],
  },
],
```

### **Conflicting Files:**

- `src/app/[lang]/user/icp/page.tsx` - Needs `CapsuleInfo`, `Capsule` types
- `src/ic/backend.ts` - Needs `BackendActor` type
- `src/services/upload/icp-upload.ts` - Needs backend types for ICP communication
- `src/services/upload/icp-with-processing.ts` - Needs backend types for ICP communication

## 🤔 **The Dilemma**

### **Option 1: Keep ESLint Rule (Domain-Driven)**

**Pros:**

- Enforces clean architecture
- Prevents direct wire type usage in app code
- Maintains separation of concerns

**Cons:**

- ICP files legitimately need backend types for direct canister communication
- Would require creating extensive domain types for all ICP operations
- Significant additional work for MVP

### **Option 2: Remove ESLint Rule (Pragmatic)**

**Pros:**

- Allows ICP files to use backend types directly
- Faster development for MVP
- Less architectural overhead

**Cons:**

- Breaks domain-driven architecture principles
- Direct wire type usage throughout codebase
- Potential type inconsistencies

### **Option 3: Hybrid Approach**

**Pros:**

- Keep rule for most files
- Allow exceptions for ICP-specific files
- Maintains most architectural benefits

**Cons:**

- Complex ESLint configuration
- Inconsistent enforcement
- Hard to maintain

## 🎯 **Context: MVP vs Greenfield**

We're in **MVP mode** but want to do the **right thing** architecturally. The question is:

1. **Should we invest time now** in creating proper domain types for ICP?
2. **Or should we be pragmatic** and allow direct wire type usage for ICP files?
3. **What's the long-term vision** for this architecture?

## 📋 **Specific Questions for Senior**

1. **Architecture Priority**: How important is maintaining strict domain-driven architecture vs. getting ICP functionality working?

2. **ICP Domain Types**: Should we create comprehensive domain types for all ICP operations (`uploads_begin`, `uploads_put_chunk`, `uploads_finish`, `memories_update`, etc.)?

3. **ESLint Strategy**: What's the recommended approach for handling legitimate wire type usage in ICP files?

4. **MVP vs Production**: Should we use different approaches for MVP vs. production code?

5. **Long-term Vision**: What's the ideal architecture for handling ICP types in the future?

## 🔧 **Current Workaround**

We've temporarily disabled the ESLint rule to unblock development:

```javascript
// Temporarily disable wire type restrictions for ICP implementation
// TODO: Re-enable after creating proper domain types for ICP
// 'no-restricted-imports': [...]
```

## 📊 **Impact Assessment**

- **Development Speed**: Current approach is faster for MVP
- **Code Quality**: Direct wire types are less clean but functional
- **Maintainability**: Hybrid approach might be harder to maintain
- **Type Safety**: All approaches maintain type safety

## 🎯 **Recommendation Request**

Please provide guidance on:

1. **Immediate approach** for unblocking ICP development
2. **Long-term strategy** for ICP type management
3. **ESLint configuration** that balances architecture and pragmatism

---

**Created**: $(date)  
**Assigned**: Senior Developer  
**Labels**: architecture, eslint, icp, mvp, decision-needed

