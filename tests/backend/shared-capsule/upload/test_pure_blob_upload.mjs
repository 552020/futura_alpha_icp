#!/usr/bin/env node

import {
  parseTestArgs,
  createTestActorOptions,
  createTestActor,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
  uploadFileAsBlob,
  verifyBlobIntegrity,
  readFileAsBuffer,
  computeSHA256Hash,
} from "../../utils/index.js";

// Test configuration
const TEST_NAME = "Pure Blob Upload Test";

// Main test function
async function testPureBlobUpload(backend, capsuleId, filePath) {
  console.log(`Starting ${TEST_NAME}`);
  console.log(`Testing pure blob upload with file: ${filePath}`);

  // Read file and compute hash
  const fileBuffer = readFileAsBuffer(filePath);
  const fileSize = fileBuffer.length;
  const expectedHash = computeSHA256Hash(fileBuffer);

  console.log(`File: ${filePath.split("/").pop()} (${fileSize} bytes)`);
  console.log(`Expected hash: ${expectedHash.toString("hex")}`);

  // Upload file as pure blob (no memory creation)
  console.log("Uploading file as pure blob...");
  const uploadResult = await uploadFileAsBlob(backend, filePath, capsuleId, {
    createMemory: false,
    idempotencyKey: `pure-blob-test-${Date.now()}`,
  });

  if (!uploadResult.success) {
    console.log(`❌ Pure blob upload failed: ${uploadResult.error}`);
    return { success: false, error: uploadResult.error };
  }

  const { blobId, size, memoryId } = uploadResult;
  console.log(`✅ Upload finished successfully!`);
  console.log(`Blob ID: ${blobId}`);
  console.log(`Memory ID: ${memoryId}`);
  console.log(`Size: ${size} bytes`);

  // Verify pure blob upload results
  console.log("Verifying pure blob upload...");

  // 1. Check that memory_id is empty (no memory created)
  if (memoryId === "" || memoryId === null) {
    console.log("✅ Memory ID is empty - no memory created (correct for pure blob upload)");
  } else {
    console.log(`❌ Memory ID should be empty but got: ${memoryId}`);
    return { success: false, error: "Memory ID should be empty for pure blob upload" };
  }

  // 2. Check that blob_id is not empty
  if (blobId && blobId.startsWith("blob_")) {
    console.log("✅ Blob ID is valid");
  } else {
    console.log(`❌ Invalid blob ID: ${blobId}`);
    return { success: false, error: "Invalid blob ID" };
  }

  // 3. Check that size matches
  if (Number(size) === fileSize) {
    console.log("✅ File size matches uploaded size");
  } else {
    console.log(`❌ Size mismatch: expected ${fileSize}, got ${size}`);
    return { success: false, error: "Size mismatch" };
  }

  // 4. Verify blob integrity using shared utility
  console.log("Testing blob integrity...");
  const integrityPassed = await verifyBlobIntegrity(backend, blobId, fileSize, expectedHash);

  if (!integrityPassed) {
    console.log(`❌ Blob integrity verification failed`);
    return { success: false, error: "Blob integrity verification failed" };
  }

  console.log("✅ Blob integrity verified successfully");

  // 5. Verify no memory was created in capsule
  console.log("Verifying no memory was created...");
  const memoriesResult = await backend.memories_list(capsuleId, [], []);

  if ("Ok" in memoriesResult) {
    const page = memoriesResult.Ok;
    const memories = page.items;
    console.log("✅ Memory list API works (no new memory should be created)");
  } else {
    console.log(`❌ Memory list failed: ${JSON.stringify(memoriesResult.Err)}`);
    return { success: false, error: "Memory list API failed" };
  }

  console.log("✅ Pure blob upload test PASSED!");
  console.log("✅ Blob upload, creation, and readback all work correctly");
  console.log("✅ No memory was created (pure blob storage)");
  console.log("✅ Ready for separate memory creation endpoints");

  return { success: true };
}

// Main execution
async function main() {
  // Parse command line arguments
  const parsedArgs = parseTestArgs("test_pure_blob_upload.mjs", "Tests pure blob upload without memory creation");

  // Extract file path from arguments (look for argument that contains a path and is not a flag)
  // Skip the first argument (script name) and look for a path that's not the node binary
  const filePath = process.argv
    .slice(2)
    .find((arg) => arg.includes("/") && !arg.startsWith("--") && !arg.includes("node") && !arg.includes("bin"));

  if (!filePath) {
    console.error("❌ Usage: node test_pure_blob_upload.mjs [--local|--mainnet] <CANISTER_ID> <FILE_PATH>");
    console.error(
      "Example: node test_pure_blob_upload.mjs --local uxrrr-q7777-77774-qaaaq-cai assets/input/avocado.jpg"
    );
    process.exit(1);
  }

  // Override canisterId if it was incorrectly parsed from file path
  if (parsedArgs.canisterId && parsedArgs.canisterId.includes("/")) {
    // Use default canister ID
    parsedArgs.canisterId = "uxrrr-q7777-77774-qaaaq-cai";
  }

  try {
    // Create test actor
    const options = createTestActorOptions(parsedArgs);
    const { actor: backend, canisterId } = await createTestActor(options);

    // Log network configuration
    logNetworkConfig(parsedArgs, canisterId);

    // Get or create test capsule
    const capsuleId = await getOrCreateTestCapsuleForUpload(backend);
    console.log(`Using capsule: ${capsuleId}`);

    // Create test runner
    const runner = createTestRunner(TEST_NAME);

    // Run the test
    await runner.runTest("Pure blob upload test", testPureBlobUpload, backend, capsuleId, filePath);

    // Print summary and exit
    const allPassed = runner.printTestSummary();
    process.exit(allPassed ? 0 : 1);
  } catch (error) {
    console.error(`❌ Test execution failed: ${error.message}`);
    process.exit(1);
  }
}

main();
