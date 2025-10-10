#!/usr/bin/env node

// ============================================================================
// ACCESS CONTROL SYSTEM - Main Entry Point
// ============================================================================

// Import the demo functions
import { runDemo } from "./demo.js";
import { helloWorld } from "./hello-world.js";
import { runSimpleObjectsDemo } from "./simple-objects.js";

console.log("🚀 Starting Access Control System Demo...\n");

try {
  // Run the hello world version first
  console.log("=".repeat(50));
  helloWorld();

  console.log("\n" + "=".repeat(50));
  console.log("Now running simple objects demo...\n");

  // Run the simple objects demo
  runSimpleObjectsDemo();

  console.log("\n" + "=".repeat(50));
  console.log("Now running the full capsule demo...\n");

  // Run the full demo
  runDemo();

  console.log("✅ All demos completed successfully!");
} catch (error) {
  console.error("❌ Demo failed:", error.message);
  console.error(error.stack);
  process.exit(1);
}
