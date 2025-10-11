/**
 * Capsule Data Creation Utilities
 *
 * Provides utilities for creating and managing test capsules
 */

/**
 * Create a test capsule
 * @param {Object} actor - Backend actor
 * @param {Object} options - Capsule creation options
 * @returns {Promise<string>} Capsule ID
 */
export async function createTestCapsule(actor, options = {}) {
  const {
    subject = null, // null for self-capsule
    idempotencyKey = null,
  } = options;

  try {
    const result = await actor.capsules_create(subject ? [subject] : []);

    if (result.Ok) {
      return result.Ok.id;
    } else {
      throw new Error(`Failed to create capsule: ${JSON.stringify(result)}`);
    }
  } catch (error) {
    throw new Error(`Failed to create test capsule: ${error.message}`);
  }
}

/**
 * Get or create a test capsule (idempotent)
 * @param {Object} actor - Backend actor
 * @param {Object} options - Options
 * @returns {Promise<string>} Capsule ID
 */
export async function getOrCreateTestCapsule(actor, options = {}) {
  try {
    // First, try to get existing capsules
    const capsules = await actor.capsules_list();

    let capsuleList;
    if (Array.isArray(capsules)) {
      capsuleList = capsules;
    } else if (capsules.Ok && Array.isArray(capsules.Ok)) {
      capsuleList = capsules.Ok;
    } else {
      capsuleList = [];
    }

    if (capsuleList.length > 0) {
      return capsuleList[0].id;
    }

    // Create a new capsule if none exists
    return await createTestCapsule(actor, options);
  } catch (error) {
    throw new Error(`Failed to get or create test capsule: ${error.message}`);
  }
}

/**
 * Get or create a test capsule using capsules_read_basic (for upload tests)
 * @param {Object} actor - Backend actor
 * @param {Object} options - Options
 * @returns {Promise<string>} Capsule ID
 */
export async function getOrCreateTestCapsuleForUpload(actor, options = {}) {
  try {
    console.log("🔍 Getting test capsule...");

    // First, try to get existing capsule using capsules_read_basic
    const capsuleResult = await actor.capsules_read_basic([]);

    if ("Ok" in capsuleResult && capsuleResult.Ok) {
      const actualCapsuleId = capsuleResult.Ok.capsule_id;
      console.log(`✅ Using existing capsule: ${actualCapsuleId}`);
      return actualCapsuleId;
    } else {
      console.log("🆕 No capsule found, creating one...");
      const createResult = await actor.capsules_create([]);

      if (!("Ok" in createResult)) {
        console.error("❌ Failed to create capsule:", createResult);
        throw new Error("Failed to create capsule: " + JSON.stringify(createResult));
      }

      const actualCapsuleId = createResult.Ok.id;
      console.log(`✅ Created new capsule: ${actualCapsuleId}`);
      return actualCapsuleId;
    }
  } catch (error) {
    throw new Error(`Failed to get or create test capsule for upload: ${error.message}`);
  }
}

/**
 * Create multiple test capsules
 * @param {Object} actor - Backend actor
 * @param {number} count - Number of capsules to create
 * @param {Object} options - Options
 * @returns {Promise<string[]>} Array of capsule IDs
 */
export async function createTestCapsulesBatch(actor, count, options = {}) {
  const capsuleIds = [];

  for (let i = 0; i < count; i++) {
    const capsuleId = await createTestCapsule(actor, {
      ...options,
      idempotencyKey: `test_capsule_${i}_${Date.now()}`,
    });
    capsuleIds.push(capsuleId);
  }

  return capsuleIds;
}

/**
 * Get capsule information
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @returns {Promise<Object>} Capsule information
 */
export async function getCapsuleInfo(actor, capsuleId) {
  try {
    const result = await actor.capsules_read_basic([capsuleId]);

    if (result.Ok) {
      return result.Ok;
    } else {
      throw new Error(`Failed to get capsule info: ${JSON.stringify(result)}`);
    }
  } catch (error) {
    throw new Error(`Failed to get capsule info: ${error.message}`);
  }
}

/**
 * List all capsules
 * @param {Object} actor - Backend actor
 * @returns {Promise<Object[]>} Array of capsules
 */
export async function listCapsules(actor) {
  try {
    const result = await actor.capsules_list();

    if (Array.isArray(result)) {
      return result;
    } else if (result.Ok && Array.isArray(result.Ok)) {
      return result.Ok;
    } else {
      return [];
    }
  } catch (error) {
    throw new Error(`Failed to list capsules: ${error.message}`);
  }
}

/**
 * Delete a test capsule
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @returns {Promise<boolean>} Success status
 */
export async function deleteTestCapsule(actor, capsuleId) {
  try {
    const result = await actor.capsules_delete(capsuleId);
    return result.Ok || false;
  } catch (error) {
    console.warn(`Failed to delete capsule ${capsuleId}: ${error.message}`);
    return false;
  }
}

/**
 * Clean up test capsules
 * @param {Object} actor - Backend actor
 * @param {string[]} capsuleIds - Array of capsule IDs to delete
 * @returns {Promise<number>} Number of capsules deleted
 */
export async function cleanupTestCapsules(actor, capsuleIds) {
  let deletedCount = 0;

  for (const capsuleId of capsuleIds) {
    if (await deleteTestCapsule(actor, capsuleId)) {
      deletedCount++;
    }
  }

  return deletedCount;
}
