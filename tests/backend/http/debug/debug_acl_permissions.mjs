/**
 * ACL Permissions Debug Test
 *
 * This test debugs the ACL system to understand why token minting fails:
 * 1. Create a memory with an image asset
 * 2. Check what access entries the memory has
 * 3. Debug the ACL adapter's can_view function
 * 4. Understand why the same identity can't mint tokens
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { createTestActor } from "../utils/core/actor.js";
import { createTestCapsule } from "../utils/helpers/capsule-creation.js";
import { createTestImageMemory } from "../utils/helpers/memory-creation.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function debugAclPermissions() {
  logHeader("🔍 Debugging ACL Permissions");

  let capsuleId = null;
  let memoryId = null;

  try {
    // Step 1: Create test actor
    logInfo("Step 1: Creating test actor...");
    const { actor } = await createTestActor();
    logSuccess("✅ Test actor created");

    // Step 2: Get current identity
    logInfo("Step 2: Getting current identity...");
    try {
      const { stdout: identityOutput } = await execAsync("dfx identity get-principal");
      const currentIdentity = identityOutput.trim();
      logSuccess(`✅ Current identity: ${currentIdentity}`);
    } catch (error) {
      logError(`❌ Failed to get identity: ${error.message}`);
    }

    // Step 3: Create capsule
    logInfo("Step 3: Creating capsule...");
    capsuleId = await createTestCapsule(actor);
    logSuccess(`✅ Capsule created: ${capsuleId}`);

    // Step 4: Create memory with image asset
    logInfo("Step 4: Creating memory with image asset...");
    memoryId = await createTestImageMemory(actor, capsuleId, {
      name: "acl_debug_image.png",
      mimeType: "image/png",
      idempotencyKey: `acl_debug_${Date.now()}`,
    });
    logSuccess(`✅ Memory created: ${memoryId}`);

    // Step 5: Read the memory and examine its structure
    logInfo("Step 5: Reading memory structure...");
    try {
      const memoryResult = await actor.memories_read(memoryId);
      if (memoryResult.Ok) {
        logSuccess("✅ Memory read successful");
        logInfo("Memory structure:");
        logInfo(`- Memory ID: ${memoryResult.Ok.id}`);
        logInfo(`- Capsule ID: ${memoryResult.Ok.capsule_id}`);
        logInfo(`- Access entries count: ${memoryResult.Ok.access_entries?.length || 0}`);

        if (memoryResult.Ok.access_entries && memoryResult.Ok.access_entries.length > 0) {
          logInfo("Access entries:");
          memoryResult.Ok.access_entries.forEach((entry, index) => {
            logInfo(`  Entry ${index + 1}:`);
            logInfo(`    - ID: ${entry.id}`);
            logInfo(`    - Role: ${entry.role}`);
            logInfo(`    - Person ref: ${entry.person_ref ? JSON.stringify(entry.person_ref) : "null"}`);
            logInfo(`    - Is public: ${entry.is_public}`);
            logInfo(`    - Grant source: ${entry.grant_source}`);
            logInfo(`    - Perm mask: ${entry.perm_mask}`);
            logInfo(`    - Condition: ${entry.condition}`);
          });
        } else {
          logError("❌ No access entries found!");
        }
      } else {
        logError(`❌ Memory read failed: ${JSON.stringify(memoryResult.Err)}`);
        throw new Error("No access to created memory");
      }
    } catch (error) {
      logError(`❌ Memory read failed: ${error.message}`);
      throw error;
    }

    // Step 6: Check capsule permissions
    logInfo("Step 6: Checking capsule permissions...");
    try {
      const { stdout: capsuleOutput } = await execAsync(
        `dfx canister call backend capsules_read '("${capsuleId}")' --output raw`
      );
      logInfo("Capsule structure:");
      logInfo(`Raw output: ${capsuleOutput.substring(0, 200)}...`);
    } catch (error) {
      logError(`❌ Failed to read capsule: ${error.message}`);
    }

    // Step 7: Try to understand why ACL fails
    logInfo("Step 7: Analyzing ACL failure...");
    logInfo("");
    logInfo("🔍 ACL Analysis:");
    logInfo("The ACL adapter's can_view function should:");
    logInfo("1. Get all accessible capsules for the caller");
    logInfo("2. Search for the memory across all accessible capsules");
    logInfo("3. Use effective_perm_mask to check VIEW permissions");
    logInfo("");
    logInfo("Possible issues:");
    logInfo("1. get_accessible_capsules() doesn't return the capsule");
    logInfo("2. get_memory() can't find the memory in the capsule");
    logInfo("3. effective_perm_mask() doesn't grant VIEW permissions");
    logInfo("4. is_owner() function has a bug");
    logInfo("");

    // Step 8: Test token minting with detailed error
    logInfo("Step 8: Testing token minting with detailed error...");
    try {
      const { stdout: tokenOutput } = await execAsync(
        `dfx canister call backend mint_http_token '("${memoryId}", vec {"thumbnail"}, null, 180)' --output raw`
      );
      const token = tokenOutput.trim();
      logSuccess(`✅ Token minted: ${token.substring(0, 50)}...`);
    } catch (tokenError) {
      logError(`❌ Token minting failed: ${tokenError.message}`);
      logInfo("");
      logInfo("🔧 Debugging steps:");
      logInfo("1. Check if the memory has proper access entries (✅ Done above)");
      logInfo("2. Verify the ACL adapter can find the memory");
      logInfo("3. Check if effective_perm_mask grants VIEW permissions");
      logInfo("4. Verify is_owner function works correctly");
    }

    return {
      success: false,
      reason: "acl_debug_completed",
      memoryId,
      analysis: "ACL debugging completed - check logs above for details",
    };
  } catch (error) {
    logError(`❌ Test failed: ${error.message}`);
    return { success: false, reason: "general_error", error: error.message };
  } finally {
    // Cleanup
    if (memoryId) {
      logInfo("Cleaning up memory...");
      try {
        await actor.memories_delete(memoryId, false);
        logSuccess("✅ Memory cleaned up");
      } catch (cleanupError) {
        logError(`❌ Cleanup failed: ${cleanupError.message}`);
      }
    }
  }
}

async function main() {
  logHeader("🚀 ACL Permissions Debug Test");

  const result = await debugAclPermissions();

  logHeader("📊 Debug Results");
  if (result.reason === "acl_debug_completed") {
    logSuccess("✅ ACL Debug Test COMPLETED");
    logInfo(`Memory ID: ${result.memoryId}`);
    logInfo(`Analysis: ${result.analysis}`);
    logInfo("");
    logInfo("🔍 Next steps:");
    logInfo("1. Check the memory access entries above");
    logInfo("2. Verify the ACL adapter logic");
    logInfo("3. Debug the effective_perm_mask function");
    logInfo("4. Check the is_owner function implementation");
  } else {
    logError(`❌ Test failed: ${result.reason.replace(/_/g, " ")}`);
    if (result.error) {
      logError(`Error: ${result.error}`);
    }
  }
}

main().catch(console.error);
