# Neon CI/CD Control

This document explains how to control Neon database operations in your CI/CD pipeline.

## Problem

The Neon CI/CD workflow automatically creates database branches for every pull request, which can:

- Block deployments if Neon operations fail
- Create unnecessary database branches for non-database changes
- Consume Neon resources and costs

## Solutions

### ✅ **Option 1: Completely Disabled (CURRENT STATUS)**

**Status: DISABLED** ✅

The Neon CI/CD workflow is currently **completely disabled**:

- ✅ **No automatic Neon operations** on pull requests
- ✅ **No deployment blocking** - Neon failures won't affect deployments
- ✅ **No skipped steps** - Clean CI/CD pipeline
- ✅ **Manual control only** - Run Neon operations when needed

**Re-enable when needed:**

```bash
./scripts/toggle-neon-workflow.sh enable
```

### Option 2: Manual Control (Current Implementation)

The current workflow has been modified to support manual control:

1. **Automatic on PRs**: Neon operations run automatically for pull requests (current behavior)
2. **Manual trigger**: You can run the workflow manually with options:
   - `skip_neon_operations=true`: Skip all Neon operations
   - `force_run=true`: Force run Neon operations even if normally skipped

### Option 3: Conditional Logic

The workflow now includes conditional logic that:

- Skips Neon operations when `skip_neon_operations=true` is set
- Allows forcing operations with `force_run=true`
- Maintains backward compatibility with existing PR behavior

## Usage Examples

### Disable for a specific deployment

```bash
# Run workflow manually with skip option
gh workflow run "Create/Delete Branch for Pull Request" \
  --field skip_neon_operations=true
```

### Force run when needed

```bash
# Run workflow manually with force option
gh workflow run "Create/Delete Branch for Pull Request" \
  --field force_run=true
```

### Completely disable Neon CI/CD

```bash
# Disable automatic Neon operations
./scripts/toggle-neon-workflow.sh disable

# Re-enable when needed
./scripts/toggle-neon-workflow.sh enable
```

## Files Modified

- `src/nextjs/.github/workflows/neon-branching.yml` - Modified with conditional logic
- `src/nextjs/.github/workflows/neon-branching-disabled.yml` - Disabled version
- `scripts/toggle-neon-workflow.sh` - Toggle script
- `docs/dev/neon-cicd-control.md` - This documentation

## Benefits

1. **No deployment blocking**: Neon failures won't block your deployments
2. **Cost control**: Only create database branches when needed
3. **Flexibility**: Choose between automatic, manual, or disabled modes
4. **Backward compatibility**: Existing PR behavior is preserved by default

## Current Status & Recommendations

### ✅ **CURRENT STATUS: DISABLED**

Your Neon CI/CD is currently **completely disabled** - this is the **recommended setup** because:

- ✅ **No deployment blocking** - Neon failures won't affect your deployments
- ✅ **No skipped steps** - Clean CI/CD pipeline without confusing "skipped" statuses
- ✅ **Cost control** - No unnecessary database branches created
- ✅ **Manual control** - Run Neon operations only when you actually need them

### When to Re-enable

Only re-enable Neon CI/CD when:

- You're working on database schema changes
- You need to test database migrations automatically
- You want automatic database branches for PRs

```bash
# Re-enable when needed
./scripts/toggle-neon-workflow.sh enable
```
