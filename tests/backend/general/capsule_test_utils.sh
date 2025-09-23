#!/bin/bash

# Capsule Test Utilities
# Shared functions for capsule-related test scripts
# 
# NOTE: This file now sources the general test_utils.sh to avoid duplication.
# All common functions are now centralized in test_utils.sh

# Source the general test utilities (which now includes all capsule functions)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"

# ============================================================================
# CAPSULE-SPECIFIC EXTENSIONS (if any)
# ============================================================================
# 
# Any capsule-specific functions that are not general enough for test_utils.sh
# should be added here. Most functions have been moved to test_utils.sh to
# avoid duplication.
