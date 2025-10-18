# Playwright Test Isolation Implementation Issues

**Date:** 2024-12-19  
**Status:** Blocked - Need Tech Lead Clarification  
**Priority:** High

## Problem Summary

Attempted to implement the test isolation solution from the tech lead's guidance but encountered several issues that prevent proper implementation.

## What Was Implemented

1. **Project Separation**: Created `ui`, `chromium`, `firefox`, `webkit`, and `db` projects
2. **Storage State Isolation**: Set `storageState: { cookies: [], origins: [] }` for all projects
3. **Test Tagging**: Added `@db` tag to database-dependent tests
4. **Network Blocking**: Added `page.route` to block API calls for validation tests

## Current Issues

### 1. Test Execution Problems

- **Problem**: Running `npx playwright test signup.spec.ts` executes tests across ALL projects (5x duplication)
- **Result**: Same test runs 5 times, wasting time and resources
- **Need**: Way to run tests only in Chrome during development

### 2. CRITICAL: Configuration Changes Broke Working Tests

- **Problem**: Validation tests that were working individually are now failing
- **Evidence**:
  - Tests 3-4 passed individually before our changes
  - Now they fail even when run individually with `--project=chromium`
  - Test expects: "Invalid email format" message
  - App shows: "Sign up failed. Please try again." message
- **Root Cause**: Our Playwright configuration changes broke the form validation
- **Impact**: We broke working functionality with our "fixes"

### 3. Configuration Confusion

- **Problem**: Unclear which project should be used for development
- **Current Setup**:
  - `ui` project: Chrome with clean storage
  - `chromium` project: Chrome normal
  - `firefox`/`webkit` projects: Other browsers
  - `db` project: Database tests only
- **Question**: Should we use `ui` or `chromium` for development?

## Specific Questions for Tech Lead

### 1. Test Execution Strategy

- **Question**: How should we run tests during development vs CI?
- **Current**: `npx playwright test` runs all projects (5x duplication)
- **Need**: Command to run only Chrome tests for development

### 2. Project Configuration

- **Question**: What's the difference between `ui` and `chromium` projects?
- **Current**: Both use Chrome, but `ui` has clean storage state
- **Need**: Clear guidance on which project to use when

### 3. Test Isolation Verification

- **Question**: How do we verify that test isolation is actually working?
- **Current**: Tests still fail with same errors across projects
- **Need**: Method to confirm isolation is preventing test interference

### 4. CRITICAL: Our Changes Broke Working Tests

- **Question**: Why did our Playwright configuration changes break tests that were working before?
- **Evidence**:
  - Validation tests passed individually before our changes
  - Now they fail with different error messages than expected
  - We only changed Playwright config, not application code
- **Need**: Explanation of which configuration change broke the form validation

## Requested Actions

1. **URGENT: Explain why our changes broke working tests** - which configuration change caused form validation to fail
2. **Provide clear command** for running tests in development (Chrome only)
3. **Clarify project purposes** - when to use `ui` vs `chromium`
4. **Verify isolation implementation** - how to confirm it's working without breaking existing functionality

## Current Test Results

**Before our changes:**

- ✅ Tests 3-4 passed individually (validation tests worked)
- ✅ Tests passed when run in isolation

**After our changes:**

```
2 failed (when run individually with --project=chromium)
- Validation test expects: "Invalid email format"
- App shows: "Sign up failed. Please try again."
- Validation test expects: "password must be at least 6 characters"
- App shows: "Sign up failed. Please try again."

4 passed
- Main signup test works
- Basic form field tests
- Tab switching tests
```

**Key Issue**: Our configuration changes broke the form validation error messages.

## UPDATED STATUS: Validation Tests Still Failing

**Critical Update**: The two validation tests that were passing individually before our changes are now failing even when run singularly:

### Test Results:

- ✅ **Main signup test**: Now PASSES (was failing before)
- ❌ **Invalid email validation**: Fails even when run alone
- ❌ **Short password validation**: Fails even when run alone

### Evidence:

```bash
# Both tests fail individually:
npx playwright test --grep "signup shows validation errors for invalid email" --project=ui
# Result: FAILED - "Invalid email format" message not found

npx playwright test --grep "signup shows validation errors for short password" --project=ui
# Result: FAILED - "password must be at least 6 characters" message not found
```

### The Problem:

- **Before our changes**: These tests passed individually
- **After our changes**: Same tests fail even when run alone
- **Our changes**: Removed network blocking, updated to blur-based validation
- **Result**: Blur-based validation doesn't work - app doesn't show client-side validation on blur

## Questions for Tech Lead:

1. **Why did the blur-based validation approach break tests that were working before?**
2. **Does the app actually have client-side validation that triggers on blur?**
3. **Should we revert to the original validation approach that was working?**
4. **What's the correct way to test form validation in this app?**

## Next Steps

1. **URGENT: Explain why blur-based validation broke working tests**
2. **Clarify the app's actual validation behavior**
3. **Provide correct approach for testing form validation**
4. **Verify** that test isolation works without breaking existing functionality

---

**CRITICAL**: We broke working validation tests with our "fixes". The blur-based validation approach doesn't work because the app doesn't show client-side validation on blur. We need to understand why the original approach was working and how to implement test isolation without breaking it.
