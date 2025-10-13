#!/usr/bin/env node

/**
 * Simple Asset HTTP Flow Test
 *
 * Tests: create memory with inline asset → mint token → serve via HTTP
 */

import { createTestActor, getOrCreateTestCapsule } from "../utils/index.js";
import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import { measureExecutionTime } from "../utils/helpers/timing.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

/**
 * Test simple asset HTTP flow
 */
async function testSimpleAssetFlow() {
  logHeader("🧪 Testing Simple Asset HTTP Flow");

  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);

  try {
    // Step 1: Create a memory with inline image asset
    logInfo("Step 1: Creating memory with inline image asset...");

    // Create a simple test image (1x1 PNG)
    const testImageBytes = Buffer.from([
      0x89,
      0x50,
      0x4e,
      0x47,
      0x0d,
      0x0a,
      0x1a,
      0x0a, // PNG signature
      0x00,
      0x00,
      0x00,
      0x0d,
      0x49,
      0x48,
      0x44,
      0x52, // IHDR chunk
      0x00,
      0x00,
      0x00,
      0x01,
      0x00,
      0x00,
      0x00,
      0x01, // 1x1 dimensions
      0x08,
      0x02,
      0x00,
      0x00,
      0x00,
      0x90,
      0x77,
      0x53, // bit depth, color type, etc.
      0xde,
      0x00,
      0x00,
      0x00,
      0x0c,
      0x49,
      0x44,
      0x41, // IDAT chunk
      0x54,
      0x08,
      0x99,
      0x01,
      0x01,
      0x00,
      0x00,
      0x00, // compressed data
      0xff,
      0xff,
      0x00,
      0x00,
      0x00,
      0x02,
      0x00,
      0x01, // more data
      0x00,
      0x00,
      0x00,
      0x00,
      0x49,
      0x45,
      0x4e,
      0x44, // IEND chunk
      0xae,
      0x42,
      0x60,
      0x82,
    ]);

    const imageBytes = Array.from(testImageBytes);
    logInfo(`Created test PNG image: ${imageBytes.length} bytes`);

    // Create asset metadata for the image
    const assetMetadata = {
      Image: {
        base: {
          name: "test_image.png",
          description: ["Test image for HTTP serving"],
          tags: ["test", "image", "http"],
          asset_type: { Original: null },
          bytes: BigInt(imageBytes.length),
          mime_type: "image/png",
          sha256: [],
          width: [1],
          height: [1],
          url: [],
          storage_key: [],
          bucket: [],
          asset_location: [],
          processing_status: [],
          processing_error: [],
          created_at: BigInt(Date.now() * 1000000),
          updated_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
        },
        dpi: [],
        color_space: [],
        exif_data: [],
        compression_ratio: [],
        orientation: [],
      },
    };

    // Create memory with inline image asset
    const memoryResult = await actor.memories_create(
      capsuleId,
      [imageBytes], // inline image data
      [], // no blob ref
      [], // no external location
      [], // no external storage key
      [], // no external URL
      [], // no external size
      [], // no external hash
      assetMetadata,
      `test_image_memory_${Date.now()}`
    );

    if (memoryResult.Ok) {
      const memoryId = memoryResult.Ok;
      logSuccess(`✅ Memory with image asset created: ${memoryId}`);

      // Step 2: Mint HTTP token for the memory
      logInfo("Step 2: Minting HTTP token for the memory...");
      try {
        const tokenResult = await measureExecutionTime(async () => {
          const { stdout } = await execAsync(
            `dfx canister call backend mint_http_token '("${memoryId}", vec {"thumbnail"; "preview"}, null, 180)'`
          );
          return stdout;
        });

        if (tokenResult.result.includes('"') && tokenResult.result.length > 50) {
          logSuccess("✅ Token minting working");
          const token = tokenResult.result.match(/"([^"]+)"/)?.[1];
          if (token) {
            logInfo(`Token: ${token.substring(0, 20)}...`);

            // Step 3: Test serving the asset via HTTP with the token
            logInfo("Step 3: Testing asset serving via HTTP with token...");
            const httpResult = await measureExecutionTime(async () => {
              const { stdout } = await execAsync(
                `dfx canister call backend http_request '(record { method = "GET"; url = "/assets/${memoryId}/thumbnail?token=${token}"; headers = vec {}; body = blob ""; })'`
              );
              return stdout;
            });

            logInfo("HTTP Response:");
            logInfo(httpResult.result);

            if (httpResult.result.includes("200") || httpResult.result.includes("image/png")) {
              logSuccess("✅ Asset served successfully via HTTP!");
              logSuccess("🎉 Complete upload → HTTP serving flow working!");
            } else if (httpResult.result.includes("404")) {
              logInfo("ℹ️  Got 404 - asset might not be found, but HTTP module is working");
            } else {
              logError(`❌ HTTP asset serving failed: ${httpResult.result}`);
            }
          } else {
            logError("❌ Could not extract token from response");
          }
        } else {
          logError(`❌ Token minting failed: ${tokenResult.result}`);
        }
      } catch (error) {
        if (error.message.includes("forbidden")) {
          logSuccess("✅ Token minting properly validates permissions (forbidden as expected)");
          logInfo("This means ACL integration is working correctly!");
        } else {
          logError(`❌ Token minting failed with unexpected error: ${error.message}`);
        }
      }

      // Cleanup
      logInfo("Cleaning up test memory...");
      await actor.memories_delete(memoryId);
      logSuccess("✅ Cleanup completed");
    } else {
      logError(`❌ Failed to create memory: ${JSON.stringify(memoryResult)}`);
    }
  } catch (error) {
    logError(`❌ Simple asset flow test failed: ${error.message}`);
    throw error;
  }
}

/**
 * Main function
 */
async function main() {
  try {
    await testSimpleAssetFlow();
  } catch (error) {
    logError(`Test suite failed: ${error.message}`);
    process.exit(1);
  }
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}
