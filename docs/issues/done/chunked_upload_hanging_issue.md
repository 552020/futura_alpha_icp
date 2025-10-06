# Chunked Upload Process Hanging Issue

## Problem Description

The chunked upload process in the file upload test (`test_memories_upload_download_file.sh`) is getting stuck during large file uploads, causing the test to hang indefinitely.

## Symptoms

1. **Test Hangs**: When uploading large files (>20KB) using the blob upload process, the test gets stuck and shows no progress
2. **High CPU Usage**: DFX process consumes 59.4% CPU and appears to be stuck on `debug_put_chunk_b64` calls
3. **No Progress Output**: The progress indicator shows no updates, suggesting the chunk upload is not completing
4. **Process Stuck**: The DFX process remains in a running state but doesn't complete the chunk upload

## Test Environment

- **Test Script**: `tests/backend/shared-capsule/memories/test_memories_upload_download_file.sh`
- **File Size**: 3.6MB (avocado_large.jpg)
- **Chunk Size**: 64KB (matches backend `CHUNK_SIZE`)
- **Total Chunks**: ~55 chunks expected
- **Backend**: ICP canister with chunked upload support

## Technical Details

### Upload Process

1. **Begin Upload**: `uploads_begin` - ✅ Works correctly
2. **Upload Chunks**: `debug_put_chunk_b64` - ❌ Gets stuck
3. **Finish Upload**: `debug_finish_hex` - Never reached

### Debug Information

```bash
# Process stuck on this call:
dfx canister call backend debug_put_chunk_b64 "(session_id, chunk_index, base64_data)"
```

### Backend Configuration

- `CHUNK_SIZE`: 64KB (65536 bytes)
- `INLINE_MAX`: 32KB (32768 bytes)
- `MAX_SINGLE_RESPONSE_SIZE`: 2MB

## Root Cause Analysis

### Key Discovery: Working Tests Use Small Chunks

After analyzing the working upload tests in `tests/backend/shared-capsule/upload/`, I found that:

- **Working tests**: Use 50-byte chunks maximum
- **Our failing test**: Uses 64KB chunks (65,536 bytes each)
- **Working "large file" test**: Only 10 chunks × 50 bytes = 500 bytes total

### Root Cause: DFX Base64 Argument Size Limit

The issue is likely **DFX command line argument size limits** when passing large base64 strings, not the backend logic.

### Potential Issues

1. **DFX Argument Size Limit**: DFX may have limits on command line argument size for base64 data
2. **Base64 Encoding Overhead**: 64KB binary → ~87KB base64 string, which might exceed DFX limits
3. **Memory Issues**: Large base64 strings in DFX process causing hangs
4. **Session Management**: Upload session might be timing out during long DFX calls

### Working vs Non-Working

- ✅ **Inline Upload**: Small files (<20KB) work perfectly
- ✅ **Upload Begin**: Session creation works
- ❌ **Chunk Upload**: Gets stuck on first or subsequent chunks
- ❌ **Upload Finish**: Never reached due to chunk upload failure

## Investigation Steps Needed

1. **Test Smaller Chunks**: Try with smaller chunk sizes (e.g., 32KB, 16KB, 8KB) to find DFX limit
2. **Check DFX Limits**: Research DFX command line argument size limits
3. **Alternative Upload Method**: Consider using `uploads_put_chunk` with blob data instead of base64
4. **Check Backend Logs**: Examine canister logs during chunk upload attempts
5. **Verify Session State**: Verify upload session remains valid during chunk uploads
6. **DFX Debug**: Run DFX with verbose logging to see where it's getting stuck

## Working Test Examples

The following tests work correctly with small chunks:

- `test_upload_workflow.sh`: Uses 50-byte chunks, completes successfully
- `test_uploads_put_chunk.sh`: Tests chunk validation with small data
- `test_upload_begin.sh`: Tests session creation and management

## Potential Solutions

1. **Reduce Chunk Size**: Use smaller chunks (e.g., 8KB or 16KB) to stay within DFX limits
2. **Use Binary Upload**: Switch from `debug_put_chunk_b64` to `uploads_put_chunk` with binary data
3. **File-based Upload**: Write chunks to temporary files and use file-based DFX calls
4. **Streaming Upload**: Implement a streaming approach that doesn't load entire chunks into memory

## Solution Implemented: Option B - Node.js Uploader

**Status**: ✅ **IMPLEMENTED**

We've implemented a Node.js uploader that bypasses DFX CLI limitations entirely by using `@dfinity/agent` directly.

### Files Created:

- `tests/backend/ic-upload.mjs` - Node.js uploader using @dfinity/agent
- `tests/backend/test-node-upload.sh` - Test script for the Node.js uploader

### How It Works:

1. **Direct Agent Communication**: Uses `@dfinity/agent` to communicate directly with the canister
2. **Binary Data**: Passes raw `Uint8Array` data instead of base64 strings
3. **No CLI Limits**: Bypasses DFX command line argument size limitations
4. **Production-Ready**: Mirrors how real clients would interact with the canister

### Usage:

```bash
# Test with large file
./tests/backend/test-node-upload.sh tests/backend/shared-capsule/memories/assets/input/avocado_large.jpg

# Or use directly
node tests/backend/ic-upload.mjs <file_path>
```

### Benefits:

- ✅ Handles large files (tested with 3.6MB files)
- ✅ Uses proper 64KB chunks as designed
- ✅ No DFX CLI limitations
- ✅ Production-ready approach
- ✅ Proper error handling and progress reporting

## Files Affected

- `tests/backend/shared-capsule/memories/test_memories_upload_download_file.sh`
- `src/backend/src/upload/blob_store.rs`
- `src/backend/backend.did` (debug endpoints)

## Priority

**High** - This blocks testing of large file uploads, which is a core feature requirement.

## Next Steps

1. Investigate backend logs during chunk upload
2. Test with different chunk sizes
3. Verify DFX argument size limits
4. Check canister memory and timeout settings
5. Consider alternative chunk upload approaches if needed

---

**Reported by**: AI Assistant  
**Date**: Current session  
**Environment**: Local development with DFX 0.29.0
