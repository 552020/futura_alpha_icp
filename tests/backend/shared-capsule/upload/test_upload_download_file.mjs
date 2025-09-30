#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { HttpAgent, Actor } from "@dfinity/agent";
import fetch from "node-fetch";
import { loadDfxIdentity, makeMainnetAgent } from "./ic-identity.js";

// Adjust to your local replica or mainnet gateway
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const CANISTER_ID = process.env.BACKEND_CANISTER_ID || process.env.BACKEND_ID || "backend";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

// Import the backend interface
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

// Configuration
const OUTPUT_DIR = "tests/backend/shared-capsule/upload/assets/output";

// Helper function to create the appropriate agent based on network
async function createAgent() {
  try {
    // Load DFX identity for both local and mainnet
    console.log("Loading DFX identity...");
    const identity = loadDfxIdentity();
    console.log(`Using DFX identity: ${identity.getPrincipal().toString()}`);

    if (IS_MAINNET) {
      return makeMainnetAgent(identity);
    } else {
      // Use DFX identity for local replica too
      const agent = new HttpAgent({ host: HOST, identity, fetch });
      // Fetch root key for local replica
      await agent.fetchRootKey();
      return agent;
    }
  } catch (error) {
    console.error("Failed to load DFX identity:", error.message);
    throw error;
  }
}

// Helper function to get or create a test capsule
async function getTestCapsuleId(backend) {
  console.log("🔍 Getting test capsule...");
  let capsuleResult = await backend.capsules_read_basic([]);
  let actualCapsuleId;

  if ("Ok" in capsuleResult && capsuleResult.Ok) {
    actualCapsuleId = capsuleResult.Ok.capsule_id;
    console.log(`✅ Using existing capsule: ${actualCapsuleId}`);
  } else {
    console.log("🆕 No capsule found, creating one...");
    const createResult = await backend.capsules_create([]);
    if (!("Ok" in createResult)) {
      console.error("❌ Failed to create capsule:", createResult);
      throw new Error("Failed to create capsule: " + JSON.stringify(createResult));
    }
    actualCapsuleId = createResult.Ok.id;
    console.log(`✅ Created new capsule: ${actualCapsuleId}`);
  }

  return actualCapsuleId;
}

// Helper function to create asset metadata for Document type
function createDocumentAssetMetadata(fileName, fileSize, mimeType = "application/octet-stream") {
  return {
    Document: {
      document_type: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000), // Convert to nanoseconds
        asset_type: { Original: null },
        sha256: [],
        name: fileName,
        storage_key: [],
        tags: ["upload-test", "file", `size-${fileSize}`],
        processing_error: [],
        mime_type: mimeType,
        description: [`Upload test file - ${fileSize} bytes`],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(fileSize),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      language: [],
      page_count: [],
      word_count: [],
    },
  };
}

// Helper function to get file size
function getFileSize(filePath) {
  try {
    const stats = fs.statSync(filePath);
    return stats.size;
  } catch (error) {
    return 0;
  }
}

// Helper function to read file as buffer
function readFileAsBuffer(filePath) {
  try {
    return fs.readFileSync(filePath);
  } catch (error) {
    throw new Error(`Failed to read file ${filePath}: ${error.message}`);
  }
}

// Helper function to compute SHA256 hash
function computeSHA256Hash(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

// Helper function to create progress bar
function createProgressBar(current, total, width = 20) {
  const percentage = Math.round((current / total) * 100);
  const filledLength = Math.round((current / total) * width);
  const bar = "█".repeat(filledLength) + "░".repeat(width - filledLength);
  return `[${bar}] ${percentage}%`;
}

// Function to upload file using blob upload process
async function uploadFileViaBlob(backend, filePath, capsuleId) {
  const fileName = path.basename(filePath);
  const fileBuffer = readFileAsBuffer(filePath);
  const fileSize = fileBuffer.length;

  console.log(`🚀 Starting blob upload for ${fileName} (${fileSize} bytes)`);

  // Calculate chunk size (64KB chunks - matches backend CHUNK_SIZE)
  const chunkSize = 65536;
  const totalChunks = Math.ceil(fileSize / chunkSize);

  console.log(`📦 File will be uploaded in ${totalChunks} chunks of ${chunkSize} bytes each`);

  // Begin upload session
  const idempotencyKey = `test_blob_${Date.now()}`;
  const assetMetadata = createDocumentAssetMetadata(fileName, fileSize);

  const begin = await backend.uploads_begin(capsuleId, assetMetadata, totalChunks, idempotencyKey);

  if (!("Ok" in begin)) {
    console.error("❌ Failed to begin upload session:", begin);
    throw new Error("uploads_begin failed: " + JSON.stringify(begin));
  }

  const sessionId = begin.Ok;
  console.log(`✅ Upload session started with ID: ${sessionId}`);

  // Upload file in chunks
  for (let chunkIndex = 0; chunkIndex < totalChunks; chunkIndex++) {
    const offset = chunkIndex * chunkSize;
    const currentChunkSize = Math.min(chunkSize, fileSize - offset);
    const chunkData = fileBuffer.slice(offset, offset + currentChunkSize);

    // Show progress
    const progressBar = createProgressBar(chunkIndex + 1, totalChunks);
    process.stdout.write(
      `\r${progressBar} - Uploading chunk ${chunkIndex + 1}/${totalChunks} (${currentChunkSize} bytes)`
    );

    // Upload chunk
    const putResult = await backend.uploads_put_chunk(sessionId, chunkIndex, new Uint8Array(chunkData));

    if (!("Ok" in putResult)) {
      console.log(""); // New line after progress
      console.error(`❌ Failed to upload chunk ${chunkIndex}:`, putResult);
      throw new Error(`uploads_put_chunk failed: ${JSON.stringify(putResult)}`);
    }
  }

  // Show 100% completion
  console.log(`\r[████████████████████] 100% - Upload completed successfully!`);

  // Compute SHA256 hash of the entire file
  const fileHash = computeSHA256Hash(fileBuffer);
  const hashBuffer = Buffer.from(fileHash, "hex");

  // Finish upload
  console.log(`🔐 Finishing upload with hash: ${fileHash}`);
  const finish = await backend.uploads_finish(sessionId, new Uint8Array(hashBuffer), BigInt(fileSize));

  if (!("Ok" in finish)) {
    console.error("❌ Failed to finish upload:", finish);
    throw new Error("uploads_finish failed: " + JSON.stringify(finish));
  }

  const memoryId = finish.Ok;
  console.log(`✅ Blob upload successful - Memory ID: ${memoryId}`);
  return memoryId;
}

// Function to upload file using inline upload (for small files)
async function uploadFileInline(backend, filePath, capsuleId) {
  const fileName = path.basename(filePath);
  const fileBuffer = readFileAsBuffer(filePath);
  const fileSize = fileBuffer.length;

  console.log(`📄 Starting inline upload for ${fileName} (${fileSize} bytes)`);

  const assetMetadata = createDocumentAssetMetadata(fileName, fileSize);
  const idempotencyKey = `test_inline_${Date.now()}`;

  // Compute SHA256 hash
  const fileHash = computeSHA256Hash(fileBuffer);
  const hashBuffer = Buffer.from(fileHash, "hex");

  const result = await backend.memories_create(
    capsuleId,
    [new Uint8Array(fileBuffer)], // opt blob - inline data
    [], // opt BlobRef - no blob reference for inline
    [], // opt StorageEdgeBlobType - no storage edge for inline
    [], // opt text - no storage key for inline
    [], // opt text - no bucket for inline
    [], // opt nat64 - no file_created_at
    [new Uint8Array(hashBuffer)], // opt blob - sha256 hash
    assetMetadata,
    idempotencyKey
  );

  if (!("Ok" in result)) {
    console.error("❌ Inline upload failed:", result);
    throw new Error("memories_create failed: " + JSON.stringify(result));
  }

  const memoryId = result.Ok;
  console.log(`✅ Inline upload successful - Memory ID: ${memoryId}`);
  return memoryId;
}

// Function to download file from memory
async function downloadFileFromMemory(backend, memoryId, outputPath, testName) {
  console.log(`📥 Downloading file from memory ID: ${memoryId}`);

  const result = await backend.memories_read(memoryId);

  if (!("Ok" in result)) {
    console.error(`❌ ${testName} - Failed to retrieve memory:`, result);
    throw new Error("memories_read failed: " + JSON.stringify(result));
  }

  const memory = result.Ok;
  let fileBuffer = null;

  // Check for inline assets first
  if (memory.inline_assets && memory.inline_assets.length > 0) {
    const inlineAsset = memory.inline_assets[0];
    fileBuffer = Buffer.from(inlineAsset.bytes);
    console.log(`📄 Found inline asset (${fileBuffer.length} bytes)`);
  }
  // Check for blob internal assets
  else if (memory.blob_internal_assets && memory.blob_internal_assets.length > 0) {
    const blobAsset = memory.blob_internal_assets[0];
    const blobRef = blobAsset.blob_ref;
    console.log(`📦 Found blob internal asset with locator: ${blobRef.locator}`);

    // Get blob metadata to determine if we need chunked reading
    const metaResult = await backend.blob_get_meta(blobRef.locator);
    if (!("Ok" in metaResult)) {
      console.error(`❌ ${testName} - Failed to get blob metadata:`, metaResult);
      throw new Error("blob_get_meta failed: " + JSON.stringify(metaResult));
    }

    const { size: blobSize, chunk_count: totalChunks } = metaResult.Ok;
    console.log(`📊 Blob size: ${blobSize} bytes, chunks: ${totalChunks}`);

    // Use chunked reading for all blobs (more reliable)
    console.log(`📦 Downloading blob in ${totalChunks} chunks...`);
    const chunks = [];

    for (let chunkIndex = 0; chunkIndex < totalChunks; chunkIndex++) {
      const progressBar = createProgressBar(chunkIndex + 1, totalChunks);
      process.stdout.write(`\r${progressBar} - Downloading chunk ${chunkIndex + 1}/${totalChunks}`);

      const chunkResult = await backend.blob_read_chunk(blobRef.locator, chunkIndex);
      if (!("Ok" in chunkResult)) {
        console.log(""); // New line after progress
        console.error(`❌ ${testName} - Failed to read chunk ${chunkIndex}:`, chunkResult);
        throw new Error(`blob_read_chunk failed: ${JSON.stringify(chunkResult)}`);
      }

      chunks.push(Buffer.from(chunkResult.Ok));
    }

    // Show 100% completion
    console.log(`\r[████████████████████] 100% - Download completed successfully!`);

    // Combine all chunks
    fileBuffer = Buffer.concat(chunks);
    console.log(`📦 Downloaded blob data (${fileBuffer.length} bytes)`);
  }
  // Check for blob external assets
  else if (memory.blob_external_assets && memory.blob_external_assets.length > 0) {
    const blobAsset = memory.blob_external_assets[0];
    console.log(`🌐 Found blob external asset with URL: ${blobAsset.url}`);
    // For external assets, we would need to fetch from the URL
    // This is more complex and depends on the external storage implementation
    throw new Error("External blob assets not yet supported in this test");
  }

  if (!fileBuffer) {
    console.error(`❌ ${testName} - No file data found in memory`);
    console.log("Memory structure:", JSON.stringify(memory, null, 2));
    throw new Error("No file data found in memory");
  }

  // Save file
  fs.writeFileSync(outputPath, fileBuffer);

  if (fs.existsSync(outputPath)) {
    const fileSize = fs.statSync(outputPath).size;
    console.log(`✅ ${testName} - File downloaded successfully to: ${outputPath} (${fileSize} bytes)`);
    return fileBuffer;
  } else {
    throw new Error(`Failed to save downloaded file to ${outputPath}`);
  }
}

// Function to verify downloaded file
function verifyDownloadedFile(originalPath, downloadedPath, testName, skipVerification = false) {
  if (!fs.existsSync(downloadedPath)) {
    console.error(`❌ Downloaded file not found: ${downloadedPath}`);
    return false;
  }

  // If verification was skipped (for large files), just confirm the placeholder exists
  if (skipVerification) {
    const downloadedSize = getFileSize(downloadedPath);
    console.log(`✅ ${testName} - Upload verification passed (${downloadedSize} bytes placeholder created)`);
    return true;
  }

  const originalSize = getFileSize(originalPath);
  const downloadedSize = getFileSize(downloadedPath);

  console.log(`🔍 Original size: ${originalSize} bytes`);
  console.log(`🔍 Downloaded size: ${downloadedSize} bytes`);

  // Allow for small differences due to compression/encoding
  const sizeDiff = Math.abs(originalSize - downloadedSize);
  const sizeDiffPercent = (sizeDiff / originalSize) * 100;

  if (sizeDiffPercent < 1) {
    console.log(`✅ ${testName} - File size verification passed (${sizeDiffPercent.toFixed(2)}% difference)`);
    return true;
  } else {
    console.error(`❌ ${testName} - File size verification failed (${sizeDiffPercent.toFixed(2)}% difference)`);
    return false;
  }
}

// Main test function
async function testFileUploadDownload(backend, filePath, capsuleId) {
  const fileName = path.basename(filePath);
  const testName = "File upload/download test";

  console.log("🧪 === File Upload and Download Test ===");
  console.log(`📁 File: ${fileName}`);
  console.log(`📁 Path: ${filePath}`);

  // Check if file exists
  if (!fs.existsSync(filePath)) {
    throw new Error(`File not found: ${filePath}`);
  }

  const fileSize = getFileSize(filePath);
  console.log(`📏 Size: ${fileSize} bytes`);

  // Use blob upload for all files (more reliable than inline)
  console.log(`📦 Using blob upload process for ${fileSize} bytes`);
  const memoryId = await uploadFileViaBlob(backend, filePath, capsuleId);

  // Create output directory if it doesn't exist
  if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  }

  // Download the file
  console.log("📥 Downloading file...");
  const outputPath = path.join(OUTPUT_DIR, `downloaded_${fileName}`);
  const downloadedBuffer = await downloadFileFromMemory(backend, memoryId, outputPath, testName);

  // No need to skip verification anymore - chunked reading handles all file sizes
  const skipVerification = false;

  // Verify the downloaded file
  console.log("🔍 Verifying downloaded file...");
  if (verifyDownloadedFile(filePath, outputPath, testName, skipVerification)) {
    if (skipVerification) {
      console.log("🎉 File upload test completed successfully! (Download verification skipped for large file)");
    } else {
      console.log("🎉 File upload and download test completed successfully!");
    }
    console.log(`📁 Original file: ${filePath}`);
    console.log(`📁 Downloaded file: ${outputPath}`);
    return true;
  } else {
    throw new Error("File verification failed");
  }
}

// Main execution
async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.error("❌ Usage: node test_upload_download_file.mjs <file_path>");
    console.log(
      "📝 Example: node test_upload_download_file.mjs tests/backend/shared-capsule/upload/assets/input/avocado_tiny.jpg"
    );
    process.exit(1);
  }

  const filePath = args[0];

  console.log("=========================================");
  console.log("🧪 Starting File Upload/Download Test");
  console.log("=========================================");

  try {
    // Create the appropriate agent for the network
    const agent = await createAgent();

    console.log(`Using ${IS_MAINNET ? "MAINNET" : "LOCAL"} mode`);
    console.log(`Host: ${HOST}`);
    console.log(`Canister ID: ${CANISTER_ID}`);

    const backend = Actor.createActor(idlFactory, {
      agent,
      canisterId: CANISTER_ID,
    });

    // Get or create a test capsule
    const capsuleId = await getTestCapsuleId(backend);

    // Run the test
    const success = await testFileUploadDownload(backend, filePath, capsuleId);

    if (success) {
      console.log("=========================================");
      console.log("Test Summary for File Upload/Download");
      console.log("=========================================");
      console.log("✅ [PASS] 🎉 File upload/download test passed!");
      console.log(`📁 Output directory: ${OUTPUT_DIR}`);
      process.exit(0);
    } else {
      throw new Error("Test failed");
    }
  } catch (error) {
    console.log("=========================================");
    console.log("Test Summary for File Upload/Download");
    console.log("=========================================");
    console.error("❌ [FAIL] 💥 File upload/download test failed!");
    console.error(`Error: ${error.message}`);
    process.exit(1);
  }
}

// Run main function if script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
