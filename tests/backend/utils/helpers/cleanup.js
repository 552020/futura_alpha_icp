/**
 * Cleanup Utilities
 *
 * Provides utilities for cleaning up test data and resources
 */

/**
 * Clean up test memories
 * @param {Object} actor - Backend actor
 * @param {string[]} memoryIds - Array of memory IDs to delete
 * @returns {Promise<number>} Number of memories deleted
 */
export async function cleanupTestMemories(actor, memoryIds) {
  let deletedCount = 0;

  for (const memoryId of memoryIds) {
    try {
      const result = await actor.memories_delete(memoryId);
      if (result.Ok) {
        deletedCount++;
      }
    } catch (error) {
      console.warn(`Failed to cleanup memory ${memoryId}: ${error.message}`);
    }
  }

  return deletedCount;
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
    try {
      const result = await actor.capsules_delete(capsuleId);
      if (result.Ok) {
        deletedCount++;
      }
    } catch (error) {
      console.warn(`Failed to cleanup capsule ${capsuleId}: ${error.message}`);
    }
  }

  return deletedCount;
}

/**
 * Clean up all test data for a capsule
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @returns {Promise<Object>} Cleanup results
 */
export async function cleanupCapsuleData(actor, capsuleId) {
  try {
    // Get all memories in the capsule
    const memories = await actor.memories_list(capsuleId);
    const memoryList = Array.isArray(memories) ? memories : memories.Ok || [];

    // Delete all memories
    const memoryIds = memoryList.map((memory) => memory.id);
    const deletedMemories = await cleanupTestMemories(actor, memoryIds);

    // Delete the capsule
    const capsuleResult = await actor.capsules_delete(capsuleId);
    const deletedCapsule = capsuleResult.Ok || false;

    return {
      deletedMemories,
      deletedCapsule,
      totalMemories: memoryIds.length,
    };
  } catch (error) {
    console.warn(`Failed to cleanup capsule data for ${capsuleId}: ${error.message}`);
    return {
      deletedMemories: 0,
      deletedCapsule: false,
      totalMemories: 0,
    };
  }
}

/**
 * Clean up all test data
 * @param {Object} actor - Backend actor
 * @returns {Promise<Object>} Cleanup results
 */
export async function cleanupAllTestData(actor) {
  try {
    // Get all capsules
    const capsules = await actor.capsules_list();
    const capsuleList = Array.isArray(capsules) ? capsules : capsules.Ok || [];

    let totalDeletedMemories = 0;
    let totalDeletedCapsules = 0;

    // Clean up each capsule
    for (const capsule of capsuleList) {
      const cleanupResult = await cleanupCapsuleData(actor, capsule.id);
      totalDeletedMemories += cleanupResult.deletedMemories;
      if (cleanupResult.deletedCapsule) {
        totalDeletedCapsules++;
      }
    }

    return {
      totalDeletedMemories,
      totalDeletedCapsules,
      totalCapsules: capsuleList.length,
    };
  } catch (error) {
    console.warn(`Failed to cleanup all test data: ${error.message}`);
    return {
      totalDeletedMemories: 0,
      totalDeletedCapsules: 0,
      totalCapsules: 0,
    };
  }
}

/**
 * Clean up test data with specific patterns
 * @param {Object} actor - Backend actor
 * @param {string[]} patterns - Array of patterns to match
 * @returns {Promise<Object>} Cleanup results
 */
export async function cleanupTestDataByPattern(actor, patterns) {
  try {
    // Get all capsules
    const capsules = await actor.capsules_list();
    const capsuleList = Array.isArray(capsules) ? capsules : capsules.Ok || [];

    let totalDeletedMemories = 0;
    let totalDeletedCapsules = 0;

    for (const capsule of capsuleList) {
      // Check if capsule matches any pattern
      const capsuleMatches = patterns.some(
        (pattern) => capsule.id.includes(pattern) || (capsule.subject && capsule.subject.includes(pattern))
      );

      if (capsuleMatches) {
        const cleanupResult = await cleanupCapsuleData(actor, capsule.id);
        totalDeletedMemories += cleanupResult.deletedMemories;
        if (cleanupResult.deletedCapsule) {
          totalDeletedCapsules++;
        }
      } else {
        // Check memories in capsule for patterns
        const memories = await actor.memories_list(capsule.id);
        const memoryList = Array.isArray(memories) ? memories : memories.Ok || [];

        const matchingMemories = memoryList.filter((memory) => patterns.some((pattern) => memory.id.includes(pattern)));

        if (matchingMemories.length > 0) {
          const memoryIds = matchingMemories.map((memory) => memory.id);
          const deletedMemories = await cleanupTestMemories(actor, memoryIds);
          totalDeletedMemories += deletedMemories;
        }
      }
    }

    return {
      totalDeletedMemories,
      totalDeletedCapsules,
      patterns,
    };
  } catch (error) {
    console.warn(`Failed to cleanup test data by pattern: ${error.message}`);
    return {
      totalDeletedMemories: 0,
      totalDeletedCapsules: 0,
      patterns,
    };
  }
}

/**
 * Clean up test data created in the last N minutes
 * @param {Object} actor - Backend actor
 * @param {number} minutes - Number of minutes to look back
 * @returns {Promise<Object>} Cleanup results
 */
export async function cleanupRecentTestData(actor, minutes = 60) {
  try {
    const cutoffTime = Date.now() - minutes * 60 * 1000;

    // Get all capsules
    const capsules = await actor.capsules_list();
    const capsuleList = Array.isArray(capsules) ? capsules : capsules.Ok || [];

    let totalDeletedMemories = 0;
    let totalDeletedCapsules = 0;

    for (const capsule of capsuleList) {
      // Check if capsule was created recently
      const capsuleTime = new Date(capsule.created_at).getTime();
      if (capsuleTime > cutoffTime) {
        const cleanupResult = await cleanupCapsuleData(actor, capsule.id);
        totalDeletedMemories += cleanupResult.deletedMemories;
        if (cleanupResult.deletedCapsule) {
          totalDeletedCapsules++;
        }
      }
    }

    return {
      totalDeletedMemories,
      totalDeletedCapsules,
      cutoffTime: new Date(cutoffTime).toISOString(),
      minutes,
    };
  } catch (error) {
    console.warn(`Failed to cleanup recent test data: ${error.message}`);
    return {
      totalDeletedMemories: 0,
      totalDeletedCapsules: 0,
      cutoffTime: null,
      minutes,
    };
  }
}

/**
 * Create a cleanup function for a specific test
 * @param {Object} actor - Backend actor
 * @param {string[]} memoryIds - Memory IDs to clean up
 * @param {string[]} capsuleIds - Capsule IDs to clean up
 * @returns {Function} Cleanup function
 */
export function createTestCleanup(actor, memoryIds = [], capsuleIds = []) {
  return async function cleanup() {
    let deletedMemories = 0;
    let deletedCapsules = 0;

    // Clean up memories
    if (memoryIds.length > 0) {
      deletedMemories = await cleanupTestMemories(actor, memoryIds);
    }

    // Clean up capsules
    if (capsuleIds.length > 0) {
      deletedCapsules = await cleanupTestCapsules(actor, capsuleIds);
    }

    return {
      deletedMemories,
      deletedCapsules,
      totalItems: memoryIds.length + capsuleIds.length,
    };
  };
}
