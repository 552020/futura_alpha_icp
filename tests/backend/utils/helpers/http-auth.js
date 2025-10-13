/**
 * HTTP Authentication Utilities
 *
 * Provides utilities for testing HTTP token-gated asset serving
 * Handles the complete flow: capsule creation → memory creation → token minting → HTTP serving
 */

import { exec } from "child_process";
import { promisify } from "util";
import { logInfo, logSuccess, logError } from "./logging.js";

const execAsync = promisify(exec);

/**
 * Create a test capsule for HTTP authentication testing using existing utilities
 * @param {Object} options - Capsule creation options
 * @returns {Promise<string>} Capsule ID
 */
export async function createTestCapsule(options = {}) {
  logInfo(`Creating test capsule for HTTP auth`);

  // Use the new capsule creation helper
  const { createTestCapsule } = await import("./capsule-creation.js");
  const { createTestActor } = await import("../core/actor.js");

  const { actor } = await createTestActor();

  try {
    const capsuleId = await createTestCapsule(actor, options);
    return capsuleId;
  } catch (error) {
    logError(`❌ Failed to create capsule: ${error.message}`);
    throw error;
  }
}

/**
 * Create a test memory with inline image data using existing utilities
 * @param {string} capsuleId - Capsule ID
 * @param {Object} options - Memory creation options
 * @returns {Promise<string>} Memory ID
 */
export async function createTestMemoryWithImage(capsuleId, options = {}) {
  const {
    name = "test_image.png",
    mimeType = "image/jpeg",
    width = 1,
    height = 1,
    idempotencyKey = `test_memory_${Date.now()}`,
  } = options;

  logInfo(`Creating test memory with image: ${name}`);

  // Use the new memory creation helper
  const { createTestImageMemory } = await import("./memory-creation.js");
  const { createTestActor } = await import("../core/actor.js");

  const { actor } = await createTestActor();

  try {
    // Use the new helper to create memory with image content
    const memoryId = await createTestImageMemory(actor, capsuleId, {
      name: name,
      mimeType: mimeType,
      idempotencyKey: idempotencyKey,
    });

    return memoryId;
  } catch (error) {
    logError(`❌ Failed to create memory: ${error.message}`);
    throw error;
  }
}

/**
 * Mint an HTTP token for a memory
 * @param {string} memoryId - Memory ID
 * @param {Array<string>} variants - Asset variants (e.g., ["thumbnail", "preview"])
 * @param {Array<string>|null} assetIds - Specific asset IDs (null for all)
 * @param {number} ttlSecs - Token TTL in seconds
 * @returns {Promise<string>} HTTP token
 */
export async function mintHttpToken(memoryId, variants = ["thumbnail"], assetIds = null, ttlSecs = 180) {
  logInfo(`Minting HTTP token for memory: ${memoryId}`);

  try {
    // Use the actor interface to ensure identity consistency
    const { createTestActor } = await import("../core/actor.js");
    const { actor } = await createTestActor();

    const token = await actor.mint_http_token(memoryId, variants, assetIds, ttlSecs);
    logSuccess(`✅ HTTP token minted: ${token.substring(0, 50)}...`);
    return token;
  } catch (error) {
    logError(`❌ Failed to mint token: ${error.message}`);
    throw error;
  }
}

/**
 * Test HTTP asset serving with a token
 * @param {string} memoryId - Memory ID
 * @param {string} token - HTTP token
 * @param {string} variant - Asset variant (e.g., "thumbnail")
 * @param {string} canisterId - Canister ID (defaults to current backend)
 * @returns {Promise<Object>} Test result with status, headers, and body
 */
export async function testHttpAssetServing(
  memoryId,
  token,
  variant = "thumbnail",
  canisterId = "uxrrr-q7777-77774-qaaaq-cai"
) {
  logInfo(`Testing HTTP asset serving: ${memoryId}/${variant}`);

  const url = `http://${canisterId}.localhost:4943/asset/${memoryId}/${variant}?token=${token}`;

  try {
    const { stdout } = await execAsync(`curl -s -i "${url}"`);

    const lines = stdout.trim().split("\n");
    const statusLine = lines.find((line) => line.startsWith("HTTP/"));
    const body = lines[lines.length - 1];

    // Extract headers
    const headers = {};
    lines.forEach((line) => {
      if (line.includes(":")) {
        const [key, value] = line.split(":", 2);
        headers[key.trim().toLowerCase()] = value.trim();
      }
    });

    const result = {
      success: statusLine && statusLine.includes("200"),
      status: statusLine,
      headers,
      body,
      url,
      bodySize: body.length,
    };

    if (result.success) {
      logSuccess(`✅ HTTP asset serving successful: ${result.status}`);
      logInfo(`Body size: ${result.bodySize} bytes`);
    } else {
      logError(`❌ HTTP asset serving failed: ${result.status}`);
    }

    return result;
  } catch (error) {
    logError(`❌ Failed to test HTTP serving: ${error.message}`);
    throw error;
  }
}

/**
 * Complete HTTP authentication flow: create capsule → memory → token → test serving
 * @param {Object} options - Configuration options
 * @returns {Promise<Object>} Complete test result
 */
export async function runCompleteHttpAuthFlow(options = {}) {
  const {
    capsuleName = "test_capsule_http_auth",
    capsuleDescription = "Test capsule for HTTP authentication",
    memoryOptions = {},
    variants = ["thumbnail"],
    assetIds = null,
    ttlSecs = 180,
    canisterId = "uxrrr-q7777-77774-qaaaq-cai",
  } = options;

  logInfo("🚀 Starting complete HTTP authentication flow");

  try {
    // Step 1: Create capsule
    const capsuleId = await createTestCapsule(capsuleName, capsuleDescription);

    // Step 2: Create memory with image
    const memoryId = await createTestMemoryWithImage(capsuleId, memoryOptions);

    // Step 3: Mint token
    const token = await mintHttpToken(memoryId, variants, assetIds, ttlSecs);

    // Step 4: Test HTTP serving
    const servingResult = await testHttpAssetServing(memoryId, token, variants[0], canisterId);

    const result = {
      success: servingResult.success,
      capsuleId,
      memoryId,
      token,
      servingResult,
      browserUrl: servingResult.url,
    };

    if (result.success) {
      logSuccess("🎉 Complete HTTP authentication flow successful!");
      logInfo(`🌐 Browser URL: ${result.browserUrl}`);
    } else {
      logError("❌ Complete HTTP authentication flow failed");
    }

    return result;
  } catch (error) {
    logError(`❌ HTTP authentication flow failed: ${error.message}`);
    throw error;
  }
}

/**
 * Clean up test resources
 * @param {string} memoryId - Memory ID to delete
 * @param {boolean} force - Force deletion
 */
export async function cleanupTestResources(memoryId, force = true) {
  logInfo(`Cleaning up test memory: ${memoryId}`);

  const cmd = `dfx canister call backend memories_delete '("${memoryId}", ${force})' --output raw`;

  try {
    await execAsync(cmd);
    logSuccess(`✅ Test memory deleted: ${memoryId}`);
  } catch (error) {
    logError(`❌ Failed to delete memory: ${error.message}`);
    // Don't throw - cleanup failures shouldn't break tests
  }
}
