#!/bin/bash

# Basic test configuration - minimal setup

# Canister IDs - dynamically read from dfx
export BACKEND_CANISTER_ID=$(dfx canister id backend 2>/dev/null)
export FRONTEND_CANISTER_ID=$(dfx canister id frontend 2>/dev/null)

# Test data configuration
export TEST_USER_PRINCIPAL=""  # Set to a test principal if needed
export TEST_TIMEOUT=30         # Timeout for dfx calls in seconds