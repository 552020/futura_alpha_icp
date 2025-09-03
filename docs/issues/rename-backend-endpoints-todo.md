# Backend Endpoint Renaming - To-Do List

## Project Overview

Rename backend functions to follow professional ICP naming conventions, consolidating the API into a clean, consistent structure.

## 📝 **Task Requirements**

### **Conventional Commit Format**

All commits must follow the conventional commit format:

```
feat(api): rename list_my_capsules to capsules_list
feat(api): rename get_capsule to capsules_read
feat(api): consolidate create_capsule and register_capsule into capsules_create
```

### **Bash Test Scripts**

Each endpoint must have its own bash test script:

- Create `scripts/test-capsules_list.sh`
- Create `scripts/test-capsules_read.sh`
- Create `scripts/test-capsules_create.sh`
- etc.

### **Testing Workflow**

For each endpoint:

1. Check frontend usage
2. Rename frontend calls if needed
3. Create bash test script
4. Test with `scripts/deploy-local.sh`
5. Compile and verify no errors
6. Commit with conventional format

## 🎯 Main Goals

- [ ] Replace amateur-sounding function names with professional ones
- [ ] Consolidate duplicate functionality (create vs register)
- [ ] Establish consistent naming patterns across all resources
- [ ] Maintain backward compatibility during transition
- [ ] Update all related documentation and frontend calls

## 📋 Phase 1: Core Functions (Week 1)

### Capsules

1. [ ] Implement `capsules_list()` - replace `list_my_capsules()`
   - [ ] Check if frontend uses `list_my_capsules()` endpoint
   - [ ] Rename frontend endpoint calls if found
   - [ ] Create bash test script for `capsules_list()` endpoint
   - [ ] Test with `scripts/deploy-local.sh`
   - [ ] Compile and verify no errors
   - [ ] Commit with conventional format: `feat(api): rename list_my_capsules to capsules_list`
2. [ ] Implement `capsules_read(id)` - replace `get_capsule(id)`
   - [ ] Check if frontend uses `get_capsule()` endpoint
   - [ ] Rename frontend endpoint calls if found
   - [ ] Create bash test script for `capsules_read()` endpoint
   - [ ] Test with `scripts/deploy-local.sh`
   - [ ] Compile and verify no errors
   - [ ] Commit with conventional format: `feat(api): rename get_capsule to capsules_read`
3. [x] Implement `capsules_create(subject: Option<PersonRef>)` - replace both `create_capsule()` and `register_capsule()`
   - [x] Check if frontend uses `create_capsule()` or `register_capsule()` endpoints
   - [x] Rename frontend endpoint calls if found
   - [x] Create bash test script for `capsules_create()` endpoint
   - [x] Test with `scripts/deploy-local.sh`
   - [x] Compile and verify no errors
   - [x] Commit with conventional format: `feat(api): consolidate create_capsule and register_capsule into capsules_create`
4. [ ] Test unified create function with both `None` and `Some(subject)` parameters
   - [ ] Test with `scripts/deploy-local.sh`
   - [ ] Compile and verify no errors
   - [ ] Commit changes
5. [x] Implement `capsules_read_basic()` and `capsules_read_full()` - replace `get_user()` endpoint
   - [x] Check if frontend uses `get_user()` endpoint
   - [x] Rename frontend endpoint calls if found
   - [x] Create bash test script for both endpoints
   - [x] Test with `scripts/deploy-local.sh`
   - [x] Compile and verify no errors
   - [x] Commit with conventional format: `feat(api): replace get_user with capsules_read_basic and capsules_read_full`

### Galleries

6. [ ] Implement `galleries_list()` - replace `get_my_galleries()`
   - [ ] Check if frontend uses `get_my_galleries()` endpoint
   - [ ] Rename frontend endpoint calls if found
   - [ ] Test with `scripts/deploy-local.sh`
   - [ ] Compile and verify no errors
   - [ ] Commit changes
7. [ ] Implement `galleries_read(id)` - replace `get_gallery_by_id(id)`
   - [ ] Check if frontend uses `get_gallery_by_id()` endpoint
   - [ ] Rename frontend endpoint calls if found
   - [ ] Test with `scripts/deploy-local.sh`
   - [ ] Compile and verify no errors
   - [ ] Commit changes
8. [ ] Implement `galleries_create(data)` - replace `store_gallery_forever(data)`
   - [ ] Check if frontend uses `store_gallery_forever()` endpoint
   - [ ] Rename frontend endpoint calls if found
   - [ ] Test with `scripts/deploy-local.sh`
   - [ ] Compile and verify no errors
   - [ ] Commit changes

### General

9. [ ] Verify `whoami()` remains unchanged (classic API function)
   - [ ] Check if frontend uses `whoami()` endpoint
   - [ ] Verify no changes needed in frontend
   - [ ] Test with `scripts/deploy-local.sh`
   - [ ] Compile and verify no errors
   - [ ] Commit changes
10. [ ] Update backend.did interface definitions
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
11. [ ] Add deprecation warnings to old functions
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes

## 📋 Phase 2: Management Functions (Week 2)

### Galleries (continued)

12. [ ] Implement `galleries_update(id, patch)` - replace `update_gallery(id, data)`
     - [ ] Check if frontend uses `update_gallery()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
13. [ ] Implement `galleries_delete(id)` - replace `delete_gallery(id)`
     - [ ] Check if frontend uses `delete_gallery()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes

### Memories

14. [ ] Implement `memories_list(capsule_id)` - replace `list_capsule_memories()`
     - [ ] Check if frontend uses `list_capsule_memories()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
15. [ ] Implement `memories_create(capsule_id, data)` - replace `add_memory_to_capsule(id, data)`
     - [ ] Check if frontend uses `add_memory_to_capsule()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
16. [ ] Implement `memories_read(id)` - replace `get_memory_from_capsule(id)`
     - [ ] Check if frontend uses `get_memory_from_capsule()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
17. [ ] Implement `memories_update(id, patch)` - replace `update_memory_in_capsule(id, data)`
     - [ ] Check if frontend uses `update_memory_in_capsule()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
18. [ ] Implement `memories_delete(id)` - replace `delete_memory_from_capsule(id)`
     - [ ] Check if frontend uses `delete_memory_from_capsule()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes

### Capsules (continued)

19. [ ] Implement `capsules_bind_neon()` - replace `mark_bound()`
     - [ ] Check if frontend uses `mark_bound()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
20. [ ] Implement `capsules_verify_nonce(nonce)` - replace `verify_nonce(nonce)`
     - [ ] Check if frontend uses `verify_nonce()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes

## 📋 Phase 3: Admin & Advanced (Week 3)

### Admin Functions

21. [ ] Implement `capsules_list_all()` - admin-only, all capsules in system
     - [ ] Check if frontend uses `capsules_list_all()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
22. [ ] Implement `capsules_list_by_owner(owner)` - admin-only, role-gated cross-account queries
     - [ ] Check if frontend uses `capsules_list_by_owner()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
23. [ ] Implement `auth_register()` - replace `register()`
     - [ ] Check if frontend uses `register()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
24. [ ] Implement `auth_nonce_verify()` - replace `verify_nonce()`
     - [ ] Check if frontend uses `verify_nonce()` endpoint
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
25. [ ] Implement personal canister creation functions
     - [ ] Check if frontend uses personal canister endpoints
     - [ ] Rename frontend endpoint calls if found
     - [ ] Test with `scripts/deploy-local.sh`
     - [ ] Compile and verify no errors
     - [ ] Commit changes
26. [ ] Implement admin personal canister management functions
    - [ ] Check if frontend uses admin personal canister endpoints
    - [ ] Rename frontend endpoint calls if found
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
26. [ ] Add proper admin role checks to all admin functions
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Error Handling

27. [ ] Implement new error schema with `deprecation: Option<Text>` field
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
28. [ ] Add comprehensive error codes and messages
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
29. [ ] Update error response formats for consistency
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Logging & Monitoring

30. [ ] Implement admin access logging for `capsules_list`
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
31. [ ] Add audit events for create/register/delete operations
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
32. [ ] Set up metrics collection for function calls, response times, error rates
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

## 📋 Phase 4: Cleanup (Week 4)

### Deprecation

33. [ ] Mark all old functions as deprecated with clear warnings
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
34. [ ] Update deprecation headers in error responses
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
35. [ ] Log deprecation warnings to stderr during development
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Documentation

36. [ ] Update API documentation with new function names
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
37. [ ] Create migration guide with old → new examples
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
38. [ ] Update README and developer guides
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Testing & Performance

39. [ ] Run comprehensive tests on new functions
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
40. [ ] Performance testing to ensure no regression
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
41. [ ] Load testing with new API structure
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

## 🔧 Technical Tasks

### Bash Test Scripts

42. [ ] Create bash test script template
    - [ ] Create `scripts/test-template.sh` with common testing functions
    - [ ] Include setup, teardown, and assertion functions
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit with conventional format: `feat(test): add bash test script template`

### Backend Changes

43. [ ] Update `lib.rs` with new function signatures
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
44. [ ] Update `capsule.rs` with renamed functions
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
45. [ ] Update `types.rs` if needed for new return types
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
46. [ ] Update Candid interface definitions
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Frontend Updates

46. [ ] Update all API calls to use new function names
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
47. [ ] Update error handling for new response formats
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
48. [ ] Test frontend integration with new backend
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Testing

49. [ ] Unit tests for all new functions
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
50. [ ] Integration tests for complete workflows
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
51. [ ] Backward compatibility tests
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
52. [ ] Admin role verification tests
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

## 🚨 Critical Decisions Needed

### Before Implementation

53. [ ] **Code Review**: Confirm `create_capsule()` vs `register_capsule()` behavior
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
54. [ ] **Admin Roles**: Define admin role structure and permissions
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
55. [ ] **Error Schema**: Finalize error response format
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
56. [ ] **Pagination**: Confirm pagination limits and sorting strategy
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### During Implementation

57. [ ] **Breaking Changes**: Decide if any changes break existing integrations
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
58. [ ] **Versioning**: Plan API versioning strategy if needed
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
59. [ ] **Rollback**: Plan rollback strategy if issues arise
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

## 📊 Success Metrics

### Functionality

60. [ ] All old functions successfully replaced
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
61. [ ] No breaking changes for existing functionality
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
62. [ ] All tests passing with new implementation
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Performance

63. [ ] No performance regression
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
64. [ ] Response times within acceptable limits
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
65. [ ] Memory usage optimized
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Developer Experience

66. [ ] API feels more professional and intuitive
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
67. [ ] Function names clearly indicate purpose
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
68. [ ] Documentation is clear and helpful
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

## 🔄 Migration Strategy

### Phase 1: Parallel Implementation

69. [ ] Keep old functions working
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
70. [ ] Add new functions alongside old ones
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
71. [ ] Test both implementations
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Phase 2: Deprecation

72. [ ] Mark old functions as deprecated
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
73. [ ] Add deprecation warnings
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
74. [ ] Encourage migration to new functions
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Phase 3: Removal

75. [ ] Remove old functions in v0.8
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
76. [ ] Update all documentation
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
77. [ ] Clean up deprecated code
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

## 📝 Notes & Questions

### Open Questions

78. [ ] Does `galleries_list_for_capsule()` serve a real purpose?
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
79. [ ] Are there other functions that could be consolidated?
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
80. [ ] Should we add rate limiting during this refactor?
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

### Future Considerations

81. [ ] Plan for additional resource types
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
82. [ ] Consider API versioning strategy
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
83. [ ] Plan for future extensions
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes

## ✅ Completion Checklist

84. [ ] All new functions implemented and tested
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
85. [ ] All old functions deprecated and removed
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
86. [ ] Frontend updated to use new API
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
87. [ ] Documentation updated
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
88. [ ] Performance verified
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
89. [ ] Migration guide created
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
90. [ ] Team trained on new API
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
91. [ ] Production deployment successful
    - [ ] Test with `scripts/deploy-local.sh`
    - [ ] Compile and verify no errors
    - [ ] Commit changes
