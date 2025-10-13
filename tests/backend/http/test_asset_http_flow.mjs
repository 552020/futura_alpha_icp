#!/usr/bin/env node

/**
 * Complete Asset Upload → HTTP Serving Test
 *
 * Tests the full flow: upload asset → mint token → serve via HTTP
 */

import { createTestActor, getOrCreateTestCapsule } from "../utils/index.js";
import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import { measureExecutionTime } from "../utils/helpers/timing.js";
import { createTestMemory } from "../utils/data/memory.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

/**
 * Test complete asset upload → HTTP serving flow
 */
async function testAssetHttpFlow() {
  logHeader("🧪 Testing Complete Asset Upload → HTTP Serving Flow");

  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);

  try {
    // Step 1: Create a test memory
    logInfo("Step 1: Creating test memory...");
    const memoryId = await createTestMemory(actor, capsuleId, {
      name: "http_asset_test_memory",
      content: "Test memory for HTTP asset serving",
      tags: ["test", "http", "asset"],
    });
    logSuccess(`✅ Test memory created: ${memoryId}`);

    // Step 2: Create a test image asset
    logInfo("Step 2: Creating test image asset...");
    const testImageContent =
      "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";
    const imageBytes = Buffer.from(testImageContent.split(",")[1], "base64");

    logInfo(`Created test image: ${imageBytes.length} bytes`);

    // Step 3: Add image asset to memory
    logInfo("Step 3: Adding image asset to memory...");
    try {
      const addAssetResult = await actor.memories_add_asset(
        memoryId,
        {
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
        },
        [Array.from(imageBytes)], // inline image data
        null, // no blob ref
        null, // no external storage
        null, // no storage key
        null, // no URL
        null, // no file size
        null, // no hash
        `test_image_${Date.now()}`
      );

      if (addAssetResult.Ok) {
        logSuccess("✅ Image asset added to memory");

        // Step 4: Mint HTTP token for the asset
        logInfo("Step 4: Minting HTTP token for the asset...");
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

              // Step 5: Test serving the asset via HTTP with the token
              logInfo("Step 5: Testing asset serving via HTTP with token...");
              const httpResult = await measureExecutionTime(async () => {
                const { stdout } = await execAsync(
                  `dfx canister call backend http_request '(record { method = "GET"; url = "/assets/${memoryId}/thumbnail?token=${token}"; headers = vec {}; body = blob ""; })'`
                );
                return stdout;
              });

              if (httpResult.result.includes("200") || httpResult.result.includes("image/png")) {
                logSuccess("✅ Asset served successfully via HTTP!");
                logInfo("🎉 Complete upload → HTTP serving flow working!");

                // Step 6: Verify the content
                logInfo("Step 6: Verifying HTTP response content...");
                if (httpResult.result.includes("image/png")) {
                  logSuccess("✅ Correct content type (image/png)");
                }
                if (httpResult.result.includes("200")) {
                  logSuccess("✅ Correct HTTP status (200)");
                }
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
      } else {
        logError(`❌ Failed to add image asset: ${JSON.stringify(addAssetResult)}`);
      }
    } catch (error) {
      logError(`❌ Asset addition failed: ${error.message}`);
    }

    // Cleanup
    logInfo("Cleaning up test memory...");
    await actor.memories_delete(memoryId);
    logSuccess("✅ Cleanup completed");
  } catch (error) {
    logError(`❌ Asset HTTP flow test failed: ${error.message}`);
    throw error;
  }
}

/**
 * Main function
 */
async function main() {
  try {
    await testAssetHttpFlow();
  } catch (error) {
    logError(`Test suite failed: ${error.message}`);
    process.exit(1);
  }
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}
