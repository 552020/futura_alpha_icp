# GOAL: Test Raw Byte Behavior in Browser

## What We're Testing
We want to see what happens when we pass raw bytes (from ICP canisters) directly to the browser without any conversion or optimization.

## Key Points
- **NO optimization** - We want to see the browser's natural reaction
- **NO conversion** - Don't convert bytes to strings, base64, or anything else
- **Raw behavior only** - Just pass the bytes and see what happens

## What We're Observing
1. **Hardcoded bytes array** → `document.getElementById("bytes").textContent = bytes;`
   - Shows: `255,216,255,224,0,16,74,70,73,70,0,1,1,0,…`
   - This is what the browser naturally does with a number array

2. **ArrayBuffer from fetch** → What does the browser do with ArrayBuffer?
   - Shows: `[object ArrayBuffer]` or similar

3. **Uint8Array from fetch** → What does the browser do with Uint8Array?
   - Shows: `255,216,255,224,0,16,…` (comma-separated numbers)

## What We're NOT Doing
- Converting to base64
- Creating data URLs
- Optimizing for display
- Making it "pretty"
- Adding CSS styling
- Converting to images

## The Test
We want to see the browser's raw reaction to different byte formats, just like an ICP canister would return them.
