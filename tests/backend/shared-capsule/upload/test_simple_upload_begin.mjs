#!/usr/bin/env node

/**
 * Simple Upload Begin Test
 * 
 * This is a minimal test to isolate the uploads_begin issue
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity } from "./ic-identity.js";
import fs from "node:fs";
import path from "node:path";

// Import the backend interface
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Simple Upload Begin Test";
const CHUNK_SIZE = 64 * 1024; // 64KB chunks

// Global backend instance
let backend;

// Helper functions
function echoInfo(message) {
  console.log(`ℹ️  ${message}`);
}

function echoPass(message) {
  console.log(`✅ ${message}`);
}

function echoFail(message) {
  console.log(`❌ ${message}`);
}

function echoError(message) {
  console.error(`💥 ${message}`);
}

// Main test function
async function testSimpleUploadBegin() {
  const fileBuffer = fs.readFileSync("./assets/input/avocado_medium.jpg");
  const fileName = path.basename("./assets/input/avocado_medium.jpg");
  
  echoInfo(`Testing upload begin with file: ${fileName} (${fileBuffer.length} bytes)`);
  
  // Get or create test capsule
  const capsuleResult = await backend.capsules_read_basic([]);
  let capsuleId;
  
  if ("Ok" in capsuleResult && capsuleResult.Ok) {
    capsuleId = capsuleResult.Ok.capsule_id;
    echoInfo(`Using existing capsule: ${capsuleId}`);
  } else {
    const createResult = await backend.capsules_create([]);
    if (!("Ok" in createResult)) {
      throw new Error("Failed to create capsule: " + JSON.stringify(createResult));
    }
    capsuleId = createResult.Ok.id;
    echoInfo(`Created new capsule: ${capsuleId}`);
  }

  // Create asset metadata
  const assetMetadata = {
    Image: {
      dpi: [],
      color_space: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000),
        asset_type: { Original: null },
        sha256: [],
        name: fileName,
        storage_key: [],
        tags: ["test", "simple-upload"],
        processing_error: [],
        mime_type: "image/jpeg",
        description: [],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(fileBuffer.length),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      exif_data: [],
      compression_ratio: [],
      orientation: [],
    }
  };

  // Calculate chunk count
  const chunkCount = Math.ceil(fileBuffer.length / CHUNK_SIZE);
  const idempotencyKey = `test-simple-${Date.now()}`;
  
  echoInfo(`Chunk count: ${chunkCount}, Idempotency key: ${idempotencyKey}`);

  // Begin upload session
  echoInfo("Calling uploads_begin...");
  const beginResult = await backend.uploads_begin(capsuleId, assetMetadata, chunkCount, idempotencyKey);
  
  echoInfo(`Upload begin result: ${beginResult.Ok ? 'Success' : 'Error'}`);

  if ("Err" in beginResult) {
    throw new Error(`Upload begin failed: ${JSON.stringify(beginResult.Err)}`);
  }

  const sessionId = beginResult.Ok;
  echoInfo(`Upload session started: ${sessionId}`);
  
  return true;
}

// Main test execution
async function main() {
  echoInfo(`Starting ${TEST_NAME}`);
  
  // Get backend canister ID
  const backendCanisterId = process.argv[2];
  if (!backendCanisterId) {
    echoError("Usage: node test_simple_upload_begin.mjs <BACKEND_CANISTER_ID>");
    process.exit(1);
  }

  // Setup agent and backend
  const identity = loadDfxIdentity();
  const agent = new HttpAgent({ 
    host: "http://127.0.0.1:4943", 
    identity,
    fetch: (await import('node-fetch')).default
  });
  await agent.fetchRootKey();
  
  backend = Actor.createActor(idlFactory, {
    agent,
    canisterId: backendCanisterId,
  });

  // Run test
  try {
    const result = await testSimpleUploadBegin();
    if (result) {
      echoPass("Simple upload begin test passed!");
      process.exit(0);
    } else {
      echoFail("Simple upload begin test failed!");
      process.exit(1);
    }
  } catch (error) {
    echoError(`Test failed: ${error.message}`);
    process.exit(1);
  }
}

// Run the test
main().catch(error => {
  echoError(`Test execution failed: ${error.message}`);
  process.exit(1);
});
