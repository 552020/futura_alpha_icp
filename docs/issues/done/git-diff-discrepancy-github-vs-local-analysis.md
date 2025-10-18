# Git Diff Discrepancy: GitHub UI vs Local Analysis

**Priority:** High  
**Assigned:** Tech Lead  
**Date:** December 2024  
**Status:** Open

## Issue Description

There is a significant discrepancy between what GitHub UI shows and what local git commands report for the branch `fix/folder-upload-parentfolderid-and-s3-unification`.

## The Problem

**GitHub UI Shows:** 3 files changed  
**Local Git Commands Show:** 349 files changed in `src/nextjs` submodule

## Commands Used

```bash
# Navigate to submodule
cd /Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs

# Check current branch
git branch --show-current
# Output: fix/folder-upload-parentfolderid-and-s3-unification

# Count files changed vs main
git diff --name-only main | wc -l
# Output: 349

# Get detailed stats
git diff --stat main
# Output: 349 files changed, 17,042 insertions, 47,690 deletions
```

## Expected Behavior

GitHub UI and local git commands should show the same number of files changed.

## Possible Causes

1. **Different Base Branch:** GitHub might be comparing against a different base than `main`
2. **Submodule Handling:** GitHub might handle submodule diffs differently
3. **Branch State:** The local branch might be in a different state than what GitHub sees
4. **Git Configuration:** Local git configuration might be affecting the diff output

## Impact

This discrepancy makes it impossible to accurately document the scope of changes in the branch analysis.

## Request

**Tech Lead:** Please investigate this discrepancy and provide the correct command to get the same result as GitHub UI (3 files changed).

## Additional Context

- Main repository: `/Users/stefano/Documents/Code/Futura/futura_alpha_icp`
- Submodule: `/Users/stefano/Documents/Code/Futura/futura_alpha_icp/src/nextjs`
- Branch: `fix/folder-upload-parentfolderid-and-s3-unification`

## Commands to Investigate

```bash
# Check what GitHub is actually comparing
git log --oneline --graph main..HEAD
git diff --name-only main
git diff --stat main

# Check if there are uncommitted changes
git status
git diff --name-only

# Check submodule status
git submodule status
```

---

**Next Steps:**

1. Tech Lead to investigate the discrepancy
2. Provide correct commands to match GitHub UI
3. Update branch analysis documentation with accurate file counts
