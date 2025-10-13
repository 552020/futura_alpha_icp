/**
 * Capsule Creation Helper Functions
 *
 * Provides utilities for creating and managing test capsules
 * Based on the working pattern from general tests
 */

import { logInfo, logSuccess, logError } from "./logging.js";

/**
 * Create a test capsule using the actor interface
 * @param {Object} actor - Backend actor
 * @param {Object} options - Capsule creation options
 * @returns {Promise<string>} Capsule ID
 */
export async function createTestCapsule(actor, options = {}) {
  const {
    subject = null, // null for self-capsule
    idempotencyKey = null,
  } = options;

  logInfo(`Creating test capsule with subject: ${subject ? "specific" : "self"}`);

  try {
    // Use the same pattern as the working general tests
    const result = await actor.capsules_create(subject ? [subject] : []);

    if ("Ok" in result && result.Ok) {
      const capsule = result.Ok;
      const capsuleId = capsule.id;
      logSuccess(`✅ Test capsule created: ${capsuleId}`);
      return capsuleId;
    } else {
      throw new Error(`Failed to create capsule: ${JSON.stringify(result)}`);
    }
  } catch (error) {
    logError(`❌ Failed to create test capsule: ${error.message}`);
    throw error;
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

    // If we have capsules, return the first one
    if (capsuleList.length > 0) {
      const existingCapsule = capsuleList[0];
      logInfo(`Using existing capsule: ${existingCapsule.id}`);
      return existingCapsule.id;
    }

    // No existing capsules, create a new one
    logInfo("No existing capsules found, creating new one...");
    return await createTestCapsule(actor, options);
  } catch (error) {
    logError(`❌ Failed to get or create test capsule: ${error.message}`);
    throw error;
  }
}

/**
 * Create a self-capsule (no subject parameter)
 * @param {Object} actor - Backend actor
 * @returns {Promise<string>} Capsule ID
 */
export async function createSelfCapsule(actor) {
  return await createTestCapsule(actor, { subject: null });
}

/**
 * Create a capsule for a specific subject
 * @param {Object} actor - Backend actor
 * @param {Object} subject - Subject (PersonRef)
 * @returns {Promise<string>} Capsule ID
 */
export async function createCapsuleForSubject(actor, subject) {
  return await createTestCapsule(actor, { subject });
}

