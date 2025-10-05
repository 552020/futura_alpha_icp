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
    const capsules = capsuleResult.Ok;
    if (capsules.length > 0) {
      actualCapsuleId = capsules[0].id;
      console.log(`✅ Using existing capsule: ${actualCapsuleId}`);
    } else {
      console.log("📝 No capsules found, creating new one...");
      const createResult = await backend.capsules_create([]); // No subject for test capsule (creates self-capsule)
      if ("Ok" in createResult) {
        actualCapsuleId = createResult.Ok.id;
        console.log(`✅ Created new capsule: ${actualCapsuleId}`);
      } else {
        throw new Error(`Failed to create capsule: ${JSON.stringify(createResult)}`);
      }
    }
  } else {
    throw new Error(`Failed to read capsules: ${JSON.stringify(capsuleResult)}`);
  }

  return actualCapsuleId;
}

// Helper function to read file as buffer
function readFileAsBuffer(filePath) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`File not found: ${filePath}`);
  }
  return fs.readFileSync(filePath);
}

// Helper function to get file size
function getFileSize(filePath) {
  const stats = fs.statSync(filePath);
  return stats.size;
}

// Helper function to compute SHA256 hash
function computeSHA256Hash(buffer) {
  const hash = crypto.createHash("sha256");
  hash.update(buffer);
  return hash.digest("hex");
}

// Helper function to create asset metadata for inline upload (Image type)
function createInlineAssetMetadata(fileName, fileSize) {
  return {
    Image: {
      dpi: [],
      color_space: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000), // Convert to nanoseconds
        asset_type: { Original: null },
        sha256: [],
        name: fileName,
        storage_key: [],
        tags: ["inline-test", "file", `size-${fileSize}`],
        processing_error: [],
        mime_type: "image/jpeg",
        description: [`Inline upload test file - ${fileSize} bytes`],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(fileSize),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      exif_data: [],
      compression_ratio: [],
      orientation: [],
    },
  };
}

// Function to upload file using inline storage (for small files)
async function uploadFileInline(backend, filePath, capsuleId) {
  const fileName = path.basename(filePath);
  const fileBuffer = readFileAsBuffer(filePath);
  const fileSize = fileBuffer.length;

  console.log(`📄 Starting inline upload for ${fileName} (${fileSize} bytes)`);

  const assetMetadata = createInlineAssetMetadata(fileName, fileSize);
  const idempotencyKey = `test_inline_${Date.now()}`;

  // Compute SHA256 hash
  const fileHash = computeSHA256Hash(fileBuffer);
  const hashBuffer = Buffer.from(fileHash, "hex");

  console.log(`🔐 Computing hash: ${fileHash}`);

  // Create memory with inline bytes
  const result = await backend.memories_create(
    capsuleId,
    [new Uint8Array(fileBuffer)], // bytes parameter for inline storage (wrapped in array for Option<Vec<u8>>)
    [], // blob_ref (empty array for None)
    [], // external_location (empty array for None)
    [], // external_storage_key (empty array for None)
    [], // external_url (empty array for None)
    [], // external_size (empty array for None)
    [], // external_hash (empty array for None)
    assetMetadata,
    idempotencyKey
  );

  if (!("Ok" in result)) {
    console.error("❌ Failed to create memory with inline storage:", result);
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
    console.error("❌ Failed to read memory:", result);
    throw new Error("memories_read failed: " + JSON.stringify(result));
  }

  const memory = result.Ok;
  console.log(`📖 Memory retrieved: ${memory.id}`);
  console.log(`📊 Memory title: ${memory.metadata?.title || "No title"}`);
  console.log(`📊 Inline assets: ${memory.inline_assets?.length || 0}`);
  console.log(`📊 Blob internal assets: ${memory.blob_internal_assets?.length || 0}`);
  console.log(`📊 Blob external assets: ${memory.blob_external_assets?.length || 0}`);

  // Check for inline assets first
  if (memory.inline_assets && memory.inline_assets.length > 0) {
    const asset = memory.inline_assets[0];
    console.log(`📦 Found inline asset: ${asset.asset_id}`);
    console.log(`📦 Asset bytes length: ${asset.bytes?.length || 0}`);

    if (asset.bytes && asset.bytes.length > 0) {
      // Save the inline bytes to file
      fs.writeFileSync(outputPath, asset.bytes);
      console.log(
        `✅ ${testName} - Inline file downloaded successfully to: ${outputPath} (${asset.bytes.length} bytes)`
      );
      return asset.bytes;
    } else {
      throw new Error("No bytes found in inline asset");
    }
  } else {
    throw new Error("No inline assets found in memory");
  }
}

// Function to verify downloaded file
function verifyDownloadedFile(originalPath, downloadedPath, testName, skipVerification = false) {
  if (!fs.existsSync(downloadedPath)) {
    console.error(`❌ Downloaded file not found: ${downloadedPath}`);
    return false;
  }

  const originalSize = getFileSize(originalPath);
  const downloadedSize = getFileSize(downloadedPath);

  console.log(`📏 Original size: ${originalSize} bytes`);
  console.log(`📏 Downloaded size: ${downloadedSize} bytes`);

  if (originalSize !== downloadedSize) {
    console.error(`❌ ${testName} - Size mismatch: original ${originalSize} vs downloaded ${downloadedSize}`);
    return false;
  }

  if (!skipVerification) {
    // Compare file contents
    const originalBuffer = readFileAsBuffer(originalPath);
    const downloadedBuffer = readFileAsBuffer(downloadedPath);

    if (!originalBuffer.equals(downloadedBuffer)) {
      console.error(`❌ ${testName} - File content mismatch`);
      return false;
    }
  }

  console.log(`✅ ${testName} - File verification successful`);
  return true;
}

// Main test function
async function testInlineUpload(filePath) {
  const fileName = path.basename(filePath);
  const testName = `Inline Upload Test: ${fileName}`;

  console.log("=========================================");
  console.log("🧪 Starting Inline Upload Test");
  console.log("=========================================");

  try {
    // Create the appropriate agent for the network
    const agent = await createAgent();
    const backend = Actor.createActor(idlFactory, {
      agent,
      canisterId: CANISTER_ID,
    });

    // Get or create test capsule
    const capsuleId = await getTestCapsuleId(backend);
    console.log(`📦 Using capsule: ${capsuleId}`);

    // Check file exists and get size
    if (!fs.existsSync(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }

    const fileSize = getFileSize(filePath);
    console.log(`📏 File: ${fileName}`);
    console.log(`📏 Size: ${fileSize} bytes`);

    // Use inline upload for small files
    console.log(`📄 Using inline upload process for ${fileSize} bytes`);
    const memoryId = await uploadFileInline(backend, filePath, capsuleId);

    // Create output directory if it doesn't exist
    if (!fs.existsSync(OUTPUT_DIR)) {
      fs.mkdirSync(OUTPUT_DIR, { recursive: true });
    }

    // Download the file
    console.log("📥 Downloading file...");
    const outputPath = path.join(OUTPUT_DIR, `downloaded_inline_${fileName}`);
    const downloadedBuffer = await downloadFileFromMemory(backend, memoryId, outputPath, testName);

    // Verify the downloaded file
    console.log("🔍 Verifying downloaded file...");
    if (verifyDownloadedFile(filePath, outputPath, testName, false)) {
      console.log("🎉 Inline upload and download test completed successfully!");
      console.log(`📁 Original file: ${filePath}`);
      console.log(`📁 Downloaded file: ${outputPath}`);
      return true;
    } else {
      console.error("❌ File verification failed");
      return false;
    }
  } catch (error) {
    console.error(`❌ ${testName} failed:`, error.message);
    return false;
  }
}

// Main execution
async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.error("❌ Usage: node test_inline_upload.mjs <file_path>");
    console.log(
      "📝 Example: node test_inline_upload.mjs tests/backend/shared-capsule/upload/assets/input/orange_small_inline.jpg"
    );
    process.exit(1);
  }

  const filePath = args[0];

  const success = await testInlineUpload(filePath);
  process.exit(success ? 0 : 1);
}

// Run the test
main().catch((error) => {
  console.error("❌ Test failed:", error);
  process.exit(1);
});
