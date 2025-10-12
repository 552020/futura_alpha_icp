/**
 * File Operations Utilities
 *
 * Shared utilities for file operations used in upload tests.
 */

import fs from "node:fs";
import crypto from "node:crypto";

/**
 * Gets the size of a file in bytes
 * @param {string} filePath - Path to the file
 * @returns {number} File size in bytes, or 0 if file doesn't exist
 */
export function getFileSize(filePath) {
  try {
    const stats = fs.statSync(filePath);
    return stats.size;
  } catch (error) {
    return 0;
  }
}

/**
 * Reads a file as a buffer
 * @param {string} filePath - Path to the file
 * @returns {Buffer} File contents as buffer
 * @throws {Error} If file cannot be read
 */
export function readFileAsBuffer(filePath) {
  try {
    return fs.readFileSync(filePath);
  } catch (error) {
    throw new Error(`Failed to read file ${filePath}: ${error.message}`);
  }
}

/**
 * Computes SHA256 hash of a buffer
 * @param {Buffer} buffer - Buffer to hash
 * @returns {string} SHA256 hash as hex string
 */
export function computeSHA256Hash(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

/**
 * Creates a visual progress bar
 * @param {number} current - Current progress value
 * @param {number} total - Total progress value
 * @param {number} width - Width of the progress bar (default: 20)
 * @returns {string} Progress bar string
 */
export function createProgressBar(current, total, width = 20) {
  const percentage = Math.round((current / total) * 100);
  const filledLength = Math.round((current / total) * width);
  const bar = "█".repeat(filledLength) + "░".repeat(width - filledLength);
  return `[${bar}] ${percentage}%`;
}

/**
 * Checks if a file exists
 * @param {string} filePath - Path to the file
 * @returns {boolean} True if file exists, false otherwise
 */
export function fileExists(filePath) {
  try {
    return fs.existsSync(filePath);
  } catch (error) {
    return false;
  }
}

/**
 * Creates a directory if it doesn't exist
 * @param {string} dirPath - Path to the directory
 * @param {boolean} recursive - Whether to create parent directories (default: true)
 */
export function ensureDirectoryExists(dirPath, recursive = true) {
  if (!fs.existsSync(dirPath)) {
    fs.mkdirSync(dirPath, { recursive });
  }
}

/**
 * Writes a buffer to a file
 * @param {string} filePath - Path to write the file
 * @param {Buffer} buffer - Buffer to write
 */
export function writeBufferToFile(filePath, buffer) {
  fs.writeFileSync(filePath, buffer);
}

/**
 * Gets file extension from a file path
 * @param {string} filePath - Path to the file
 * @returns {string} File extension (including the dot)
 */
export function getFileExtension(filePath) {
  const ext = filePath.split(".").pop();
  return ext ? `.${ext}` : "";
}

/**
 * Gets file name without extension from a file path
 * @param {string} filePath - Path to the file
 * @returns {string} File name without extension
 */
export function getFileNameWithoutExtension(filePath) {
  const fileName = filePath.split("/").pop();
  const lastDotIndex = fileName.lastIndexOf(".");
  return lastDotIndex > 0 ? fileName.substring(0, lastDotIndex) : fileName;
}


