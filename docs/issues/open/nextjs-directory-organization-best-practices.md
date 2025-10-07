# Next.js Directory Organization Best Practices

## Issue Summary

We need to establish clear guidelines and rationale for organizing functions and files between the `services`, `lib`, `utils`, and `workers` directories in our Next.js application.

## Current State

Our Next.js app has the following key directories:

- **`/lib`** - 38 files including utilities, storage providers, auth helpers, and core functionality
- **`/services`** - 17 files focused on business logic (memories, gallery, upload, capsule)
- **`/utils`** - 8 files with helper functions and utilities
- **`/workers`** - 1 file for image processing
- **`/ic`** - ICP-specific functionality and declarations

## Questions for Tech Lead

1. **What is the clear distinction between `lib` and `utils`?**

   - Currently both contain utility functions
   - `lib` seems to have more complex, domain-specific utilities
   - `utils` appears to have simpler, more generic helpers

2. **When should business logic go in `services` vs `lib`?**

   - `services` contains domain-specific business logic (memories, gallery, upload)
   - `lib` contains infrastructure and cross-cutting concerns
   - What's the decision criteria?

3. **How do we handle shared utilities that could fit in multiple categories?**

   - Authentication utilities are in both `lib/auth-utils.ts` and `utils/authentication.ts`
   - Storage utilities are scattered across `lib/storage/` and `services/upload/`

4. **What's the rationale for the current organization?**
   - Is it based on complexity, reusability, or domain boundaries?
   - Should we refactor to follow a more consistent pattern?

## Proposed Structure Discussion

We should discuss:

- **`lib/`** - Core infrastructure, cross-cutting concerns, complex utilities
- **`services/`** - Business logic, domain-specific operations
- **`utils/`** - Simple, pure functions, generic helpers
- **`workers/`** - Web Workers for heavy computations

## Action Items

- [ ] Define clear criteria for each directory
- [ ] Document the rationale and guidelines
- [ ] Plan refactoring of misplaced files
- [ ] Create naming conventions and patterns
- [ ] Update team documentation

## Priority

**Medium** - This affects code organization and maintainability but doesn't block current development.
