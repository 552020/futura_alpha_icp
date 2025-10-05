/**
 * State Validation Utilities
 *
 * Provides utilities for validating system state after operations
 */

/**
 * Verify that memories are deleted
 * @param {Object} actor - Backend actor
 * @param {string[]} memoryIds - Array of memory IDs to check
 * @returns {Promise<boolean>} True if all memories are deleted
 */
export async function verifyMemoriesDeleted(actor, memoryIds) {
  for (const memoryId of memoryIds) {
    const readResult = await actor.memories_read(memoryId);
    if (readResult.Ok) {
      return false; // Memory still exists
    }
  }
  return true; // All memories are deleted
}

/**
 * Verify that memories exist
 * @param {Object} actor - Backend actor
 * @param {string[]} memoryIds - Array of memory IDs to check
 * @returns {Promise<boolean>} True if all memories exist
 */
export async function verifyMemoriesExist(actor, memoryIds) {
  for (const memoryId of memoryIds) {
    const readResult = await actor.memories_read(memoryId);
    if (!readResult.Ok) {
      return false; // Memory doesn't exist
    }
  }
  return true; // All memories exist
}

/**
 * Verify that capsules are deleted
 * @param {Object} actor - Backend actor
 * @param {string[]} capsuleIds - Array of capsule IDs to check
 * @returns {Promise<boolean>} True if all capsules are deleted
 */
export async function verifyCapsulesDeleted(actor, capsuleIds) {
  for (const capsuleId of capsuleIds) {
    const readResult = await actor.capsules_read_basic([capsuleId]);
    if (readResult.Ok) {
      return false; // Capsule still exists
    }
  }
  return true; // All capsules are deleted
}

/**
 * Verify that capsules exist
 * @param {Object} actor - Backend actor
 * @param {string[]} capsuleIds - Array of capsule IDs to check
 * @returns {Promise<boolean>} True if all capsules exist
 */
export async function verifyCapsulesExist(actor, capsuleIds) {
  for (const capsuleId of capsuleIds) {
    const readResult = await actor.capsules_read_basic([capsuleId]);
    if (!readResult.Ok) {
      return false; // Capsule doesn't exist
    }
  }
  return true; // All capsules exist
}

/**
 * Verify capsule has no memories
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID to check
 * @returns {Promise<boolean>} True if capsule has no memories
 */
export async function verifyCapsuleEmpty(actor, capsuleId) {
  const memories = await actor.memories_list(capsuleId);
  const memoryList = Array.isArray(memories) ? memories : memories.Ok || [];
  return memoryList.length === 0;
}

/**
 * Verify capsule has specific number of memories
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID to check
 * @param {number} expectedCount - Expected memory count
 * @returns {Promise<boolean>} True if capsule has expected count
 */
export async function verifyCapsuleMemoryCount(actor, capsuleId, expectedCount) {
  const memories = await actor.memories_list(capsuleId);
  const memoryList = Array.isArray(memories) ? memories : memories.Ok || [];
  return memoryList.length === expectedCount;
}

/**
 * Verify memory has no assets
 * @param {Object} actor - Backend actor
 * @param {string} memoryId - Memory ID to check
 * @returns {Promise<boolean>} True if memory has no assets
 */
export async function verifyMemoryNoAssets(actor, memoryId) {
  const memory = await actor.memories_read(memoryId);
  if (!memory.Ok) {
    return false; // Memory doesn't exist
  }

  const memoryData = memory.Ok;
  return (
    memoryData.inline_assets.length === 0 &&
    memoryData.blob_internal_assets.length === 0 &&
    memoryData.blob_external_assets.length === 0
  );
}

/**
 * Verify memory has specific number of assets
 * @param {Object} actor - Backend actor
 * @param {string} memoryId - Memory ID to check
 * @param {number} expectedCount - Expected asset count
 * @returns {Promise<boolean>} True if memory has expected asset count
 */
export async function verifyMemoryAssetCount(actor, memoryId, expectedCount) {
  const memory = await actor.memories_read(memoryId);
  if (!memory.Ok) {
    return false; // Memory doesn't exist
  }

  const memoryData = memory.Ok;
  const totalAssets =
    memoryData.inline_assets.length + memoryData.blob_internal_assets.length + memoryData.blob_external_assets.length;

  return totalAssets === expectedCount;
}

/**
 * Verify memory has specific asset types
 * @param {Object} actor - Backend actor
 * @param {string} memoryId - Memory ID to check
 * @param {Object} expectedAssets - Expected asset counts by type
 * @returns {Promise<boolean>} True if memory has expected asset types
 */
export async function verifyMemoryAssetTypes(actor, memoryId, expectedAssets) {
  const memory = await actor.memories_read(memoryId);
  if (!memory.Ok) {
    return false; // Memory doesn't exist
  }

  const memoryData = memory.Ok;

  if (expectedAssets.inline !== undefined && memoryData.inline_assets.length !== expectedAssets.inline) {
    return false;
  }

  if (expectedAssets.internal !== undefined && memoryData.blob_internal_assets.length !== expectedAssets.internal) {
    return false;
  }

  if (expectedAssets.external !== undefined && memoryData.blob_external_assets.length !== expectedAssets.external) {
    return false;
  }

  return true;
}

/**
 * Verify system state after bulk operation
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {Object} expectedState - Expected system state
 * @returns {Promise<boolean>} True if system state matches expected
 */
export async function verifySystemState(actor, capsuleId, expectedState) {
  const { memoryCount = null, capsuleExists = true, memoriesExist = [], memoriesDeleted = [] } = expectedState;

  // Check capsule existence
  if (capsuleExists !== null) {
    const capsuleExistsResult = await verifyCapsulesExist(actor, [capsuleId]);
    if (capsuleExistsResult !== capsuleExists) {
      return false;
    }
  }

  // Check memory count
  if (memoryCount !== null) {
    const memoryCountResult = await verifyCapsuleMemoryCount(actor, capsuleId, memoryCount);
    if (!memoryCountResult) {
      return false;
    }
  }

  // Check specific memories exist
  if (memoriesExist.length > 0) {
    const memoriesExistResult = await verifyMemoriesExist(actor, memoriesExist);
    if (!memoriesExistResult) {
      return false;
    }
  }

  // Check specific memories deleted
  if (memoriesDeleted.length > 0) {
    const memoriesDeletedResult = await verifyMemoriesDeleted(actor, memoriesDeleted);
    if (!memoriesDeletedResult) {
      return false;
    }
  }

  return true;
}
