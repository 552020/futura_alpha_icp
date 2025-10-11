/**
 * Test Framework - Main Exports
 *
 * Comprehensive test utilities for ICP backend testing
 * Consolidates all existing utilities into a unified framework
 */

// Core utilities
export * from "./core/agent.js";
export * from "./core/actor.js";
export * from "./core/identity.js";

// Data creation utilities
export * from "./data/capsule.js";
export * from "./data/memory.js";
export * from "./data/assets.js";
export * from "./data/fixtures.js";

// Validation utilities
export * from "./validation/results.js";
export * from "./validation/state.js";
export * from "./validation/errors.js";

// Helper utilities
export * from "./helpers/logging.js";
export * from "./helpers/timing.js";
export * from "./helpers/cleanup.js";
export * from "./helpers/args.js";
export * from "./helpers/test-runner.js";
export * from "./helpers/asset-metadata.js";
export * from "./helpers/file-operations.js";
export * from "./helpers/memory-creation.js";
export * from "./helpers/upload-download.js";
export * from "./helpers/verification.js";
export * from "./helpers/image-processing.js";
export * from "./helpers/asset-addition.js";

// Re-export existing utilities for backward compatibility
export * from "../shared-capsule/upload/helpers.mjs";
export * from "../shared-capsule/upload/ic-identity.js";
