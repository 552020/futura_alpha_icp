/**
 * Command Line Argument Parser Utilities
 *
 * Provides standardized command line argument parsing for all tests
 */

/**
 * Show usage information for tests
 * @param {string} testName - Name of the test
 * @param {string} description - Description of what the test does
 * @param {Array} availableTests - Array of available test names (optional)
 */
export function showTestUsage(testName, description = "", availableTests = []) {
  const testSelectionHelp =
    availableTests.length > 0
      ? `
  --test <name>          Run specific test by name
  --list-tests           List all available tests

Available Tests:
${availableTests.map((test) => `  - ${test}`).join("\n")}

Test Selection Examples:
  node ${testName} --test "test name"              # Run specific test
  node ${testName} --test "test name" --local      # Run specific test on local
  node ${testName} --list-tests                    # List all tests
`
      : "";

  console.log(`
Usage: node ${testName} [OPTIONS] [CANISTER_ID]

${description ? `${description}\n` : ""}Options:
  --local, --localnet    Use local DFX replica (default)
  --mainnet, --main      Use mainnet
  --help, -h             Show this help message${testSelectionHelp}
Examples:
  node ${testName}                                    # Local with default canister
  node ${testName} --local uxrrr-q7777-77774-qaaaq-cai # Local with specific canister
  node ${testName} --mainnet your-mainnet-canister-id  # Mainnet with specific canister
  node ${testName} --help                              # Show this help

Environment Variables:
  BACKEND_CANISTER_ID    Default canister ID
  IC_HOST               Override host (http://127.0.0.1:4943 or https://ic0.app)
`);
}

/**
 * Parse command line arguments for tests
 * @param {string} testName - Name of the test (for help display)
 * @param {string} description - Description of the test
 * @param {Array} availableTests - Array of available test names (optional)
 * @returns {Object} Parsed arguments
 */
export function parseTestArgs(testName, description = "", availableTests = []) {
  const args = process.argv.slice(2);
  let canisterId = null;
  let network = "local"; // default to local
  let selectedTest = null;
  let listTests = false;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];

    if (arg === "--mainnet" || arg === "--main") {
      network = "mainnet";
    } else if (arg === "--local" || arg === "--localnet") {
      network = "local";
    } else if (arg === "--test") {
      if (i + 1 < args.length) {
        selectedTest = args[i + 1];
        i++; // Skip the next argument as it's the test name
      } else {
        console.error("Error: --test requires a test name");
        process.exit(1);
      }
    } else if (arg === "--list-tests") {
      listTests = true;
    } else if (arg === "--help" || arg === "-h") {
      showTestUsage(testName, description, availableTests);
      process.exit(0);
    } else if (!arg.startsWith("--")) {
      // First non-flag argument is the canister ID
      canisterId = arg;
    }
  }

  // Handle --list-tests
  if (listTests) {
    if (availableTests.length === 0) {
      console.log("No tests available for selection in this test file.");
    } else {
      console.log("Available tests:");
      availableTests.forEach((test) => console.log(`  - ${test}`));
    }
    process.exit(0);
  }

  return {
    canisterId: canisterId || process.env.BACKEND_CANISTER_ID || process.env.BACKEND_ID || "backend",
    network,
    host: network === "mainnet" ? "https://ic0.app" : "http://127.0.0.1:4943",
    selectedTest,
    availableTests,
  };
}

/**
 * Create test actor options based on parsed arguments
 * @param {Object} parsedArgs - Parsed command line arguments
 * @returns {Object} Options for createTestActor
 */
export function createTestActorOptions(parsedArgs) {
  return {
    canisterId: parsedArgs.canisterId,
    host: parsedArgs.host,
  };
}

/**
 * Log network configuration
 * @param {Object} parsedArgs - Parsed command line arguments
 * @param {string} canisterId - Actual canister ID being used
 */
export function logNetworkConfig(parsedArgs, canisterId) {
  console.log(`Using ${parsedArgs.network.toUpperCase()} mode`);
  console.log(`Host: ${parsedArgs.host}`);
  console.log(`Canister ID: ${canisterId}`);
}
