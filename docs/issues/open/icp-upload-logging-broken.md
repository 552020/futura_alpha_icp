# ICP Upload Logging Broken After Flow Change

## Problem

The ICP upload logging is no longer working after changing from `uploadFileToICP` to `uploadToICPWithProcessing` in the single file processor.

## Root Cause

The logging was implemented in `uploadFileToICP` function with extensive `console.log` statements, but the new flow uses `uploadToICPWithProcessing` which calls `uploadFileToICPWithProgress` instead. This function uses `logger.info()` calls instead of `console.log()` statements.

## Current State

- Hardcoded ICP preferences log appears: "HARDCODED ICP PREFERENCES ACTIVE"
- No detailed ICP upload logs appear
- Upload process is not visible in console

## Technical Details

**Old Flow:**

```
single-file-processor.ts → uploadFileToICP() → console.log statements
```

**New Flow:**

```
single-file-processor.ts → uploadToICPWithProcessing() → uploadFileToICPWithProgress() → logger.info() statements
```

## Impact

- Cannot debug ICP upload process
- Cannot see upload progress
- Cannot identify where upload fails
- Development workflow is broken

## Solution Options

1. **Add console.log statements to uploadFileToICPWithProgress**
2. **Fix logger configuration to show ICP upload logs**
3. **Revert to uploadFileToICP for debugging**
4. **Create unified logging approach**

## Files Affected

- `src/nextjs/src/services/upload/single-file-processor.ts`
- `src/nextjs/src/services/upload/icp-with-processing.ts`
- `src/nextjs/src/services/upload/icp-upload.ts`

## Priority

High - blocks development and debugging of ICP upload functionality.

