#!/bin/bash

# Script to toggle Neon CI/CD workflow on/off
# Usage: ./scripts/toggle-neon-workflow.sh [enable|disable]

set -e

WORKFLOW_DIR="src/nextjs/.github/workflows"
ENABLED_WORKFLOW="$WORKFLOW_DIR/neon-branching.yml"
DISABLED_WORKFLOW="$WORKFLOW_DIR/neon-branching-disabled.yml"

if [ "$1" = "disable" ]; then
    echo "Disabling Neon CI/CD workflow..."
    if [ -f "$ENABLED_WORKFLOW" ]; then
        mv "$ENABLED_WORKFLOW" "$ENABLED_WORKFLOW.backup"
        echo "✅ Moved active workflow to backup"
    fi
    if [ -f "$DISABLED_WORKFLOW" ]; then
        cp "$DISABLED_WORKFLOW" "$ENABLED_WORKFLOW"
        echo "✅ Activated disabled workflow (manual trigger only)"
    fi
    echo "Neon operations are now disabled. They will only run when manually triggered."
    
elif [ "$1" = "enable" ]; then
    echo "Enabling Neon CI/CD workflow..."
    if [ -f "$ENABLED_WORKFLOW.backup" ]; then
        mv "$ENABLED_WORKFLOW.backup" "$ENABLED_WORKFLOW"
        echo "✅ Restored original workflow"
    else
        echo "❌ No backup found. Please restore the original workflow manually."
        exit 1
    fi
    echo "Neon operations are now enabled for pull requests."
    
else
    echo "Usage: $0 [enable|disable]"
    echo ""
    echo "Commands:"
    echo "  disable  - Disable Neon CI/CD (only manual triggers)"
    echo "  enable   - Enable Neon CI/CD (automatic on PRs)"
    echo ""
    echo "Current status:"
    if [ -f "$ENABLED_WORKFLOW" ] && [ ! -f "$ENABLED_WORKFLOW.backup" ]; then
        echo "  Neon CI/CD is ENABLED (automatic on PRs)"
    elif [ -f "$ENABLED_WORKFLOW.backup" ]; then
        echo "  Neon CI/CD is DISABLED (manual trigger only)"
    else
        echo "  Status unknown"
    fi
fi

