# Neon CLI Guide

## Overview

The Neon CLI (`neonctl`) is a command-line tool for managing Neon database branches, projects, and operations. It provides programmatic access to Neon's serverless PostgreSQL platform features.

## Installation

### Prerequisites

- Node.js (version 16 or higher)
- npm or yarn package manager

### Install Neon CLI

```bash
npm install -g neonctl
# or
yarn global add neonctl
```

### Verify Installation

```bash
neon --version
```

## Authentication

### Initial Setup

```bash
neon auth
```

This will open your browser for OAuth authentication with Neon.

### Check Authentication Status

```bash
neon me
```

### List Organizations

```bash
neon orgs list
```

### Set Organization Context

```bash
neon set-context --org-id <organization-id>
```

## Project Management

### List Projects

```bash
neon projects list
```

### Get Project Details

```bash
neon projects get <project-id>
```

### Create New Project

```bash
neon projects create <project-name>
```

## Branch Management

### List All Branches

```bash
neon branches list
```

### Create New Branch

```bash
neon branches create <branch-name>
```

### Get Branch Details

```bash
neon branches get <branch-name>
```

### Delete Branch

```bash
neon branches delete <branch-name>
```

### Switch to Branch

```bash
neon branches switch <branch-name>
```

## Database Operations

### Connect to Database

```bash
neon sql <branch-name>
```

### Run SQL Commands

```bash
neon sql <branch-name> --command "SELECT version();"
```

### Execute SQL File

```bash
neon sql <branch-name> --file script.sql
```

## Connection Information

### Get Connection String

```bash
neon connection-string <branch-name>
```

### Get Connection Details

```bash
neon connection-details <branch-name>
```

## Environment Variables

### Set Environment Variables

```bash
neon env set <key> <value>
```

### List Environment Variables

```bash
neon env list
```

### Get Environment Variable

```bash
neon env get <key>
```

## Monitoring and Logs

### View Logs

```bash
neon logs <branch-name>
```

### Monitor Performance

```bash
neon metrics <branch-name>
```

## Real-World Example: Project Setup

### Typical Neon Configuration

Based on a typical Neon setup:

**Organizations:**

- `org-abc-12345678` - your-email@example.com
- `org-xyz-87654321` - Team: Project Name

**Projects:**

- `project-abc-123456` - project-name-1
- `project-xyz-789012` - your-main-project
- `project-def-345678` - another-project

**Branches in your-main-project:**

- `main` (default) - Production
- `dev/feature-name` - Development
- `staging` - Staging
- `preview/pr-123-feature-branch`
- `preview/pr-124-another-feature`
- `preview/pr-125-yet-another-feature`

### Setting Up Context for Your Project

```bash
# 1. Authenticate
neon auth

# 2. List organizations
neon orgs list

# 3. Set context to your organization
neon set-context --org-id org-xyz-87654321

# 4. List projects
neon projects list

# 5. Set context to your project
neon set-context --project-id project-xyz-789012

# 6. List all branches
neon branches list
```

## Common Workflows

### 1. Development Branch Workflow

```bash
# Create a new development branch
neon branches create dev-feature-xyz

# Get connection string for the new branch
neon connection-string dev-feature-xyz

# Switch to the branch
neon branches switch dev-feature-xyz

# Run migrations
neon sql dev-feature-xyz --file migrations/001_initial.sql
```

### 2. Working with Your Branches

```bash
# Switch to development branch
neon branches switch dev/feature-name

# Get connection string for development
neon connection-string dev/feature-name

# Switch to main (production)
neon branches switch main

# Get connection string for production
neon connection-string main

# Work with a preview branch
neon branches switch preview/pr-123-feature-branch
```

### 3. Production Deployment

```bash
# List all branches
neon branches list

# Get production branch connection string
neon connection-string main

# Monitor production logs
neon logs main
```

### 4. Database Schema Management

```bash
# Create schema on specific branch
neon sql <branch-name> --command "CREATE SCHEMA IF NOT EXISTS app_schema;"

# Run Drizzle migrations
neon sql <branch-name> --file src/db/migrations/0001_initial.sql
```

## Integration with Next.js

### Environment Configuration

Create different `.env` files for different branches:

```bash
# .env.local (main branch)
DATABASE_URL=postgresql://user:pass@ep-main-xyz-pooler.region.aws.neon.tech/db

# .env.development (dev branch)
DATABASE_URL=postgresql://user:pass@ep-dev-xyz-pooler.region.aws.neon.tech/db

# .env.staging (staging branch)
DATABASE_URL=postgresql://user:pass@ep-staging-xyz-pooler.region.aws.neon.tech/db
```

### Drizzle Integration

```bash
# Generate migrations
npx drizzle-kit generate

# Apply migrations to specific branch
neon sql <branch-name> --file src/db/migrations/0001_initial.sql
```

## Branch Naming Conventions

### Recommended Patterns

- `main` - Production branch
- `staging` - Staging environment
- `dev-<feature>` - Feature development
- `preview-pr-<number>` - Pull request previews

### Example Branch Structure

```
main (production)
├── staging
├── dev-auth-system
├── dev-file-upload
├── preview-pr-123
└── preview-pr-124
```

## Troubleshooting

### Common Issues

#### 1. Authentication Errors

```bash
# Re-authenticate
neon auth --force

# Check authentication status
neon me
```

#### 2. Project Not Found

```bash
# List available projects
neon projects list

# Check current project context
neon config

# If no projects found, check organizations
neon orgs list

# Set correct organization context
neon set-context --org-id org-xyz-87654321

# Then list projects again
neon projects list
```

#### 3. Branch Connection Issues

```bash
# Verify branch exists
neon branches list

# Check branch status
neon branches get <branch-name>

# Get fresh connection string
neon connection-string <branch-name>
```

### Debug Mode

```bash
# Enable verbose logging
neon --verbose <command>

# Example
neon --verbose branches list
```

## Advanced Features

### Branch Branching

```bash
# Create branch from another branch
neon branches create new-feature --parent main
```

### Branch Reset

```bash
# Reset branch to parent state
neon branches reset <branch-name>
```

### Branch Merge

```bash
# Merge changes between branches
neon branches merge <source-branch> <target-branch>
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Database Migration
on:
  push:
    branches: [main]

jobs:
  migrate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: "18"
      - name: Install Neon CLI
        run: npm install -g neonctl
      - name: Authenticate
        run: neon auth --token ${{ secrets.NEON_TOKEN }}
      - name: Run Migrations
        run: neon sql main --file migrations/latest.sql
```

## Best Practices

### 1. Branch Management

- Use descriptive branch names
- Keep branches short-lived
- Delete unused branches regularly
- Use consistent naming conventions

### 2. Security

- Never commit connection strings to version control
- Use environment variables for sensitive data
- Rotate credentials regularly
- Use least-privilege access

### 3. Performance

- Monitor branch resource usage
- Use connection pooling
- Optimize queries before deployment
- Clean up test data regularly

## Quick Reference for Your Project

### Example Project IDs

- **Organization**: `org-xyz-87654321` (Team: Project Name)
- **Project**: `project-xyz-789012` (your-main-project)

### Quick Setup Commands

```bash
# Set context to your project
neon set-context --org-id org-xyz-87654321
neon set-context --project-id project-xyz-789012

# List your branches
neon branches list

# Get connection string for main branch
neon connection-string main

# Get connection string for development
neon connection-string dev/feature-name
```

## Useful Commands Reference

```bash
# Authentication
neon auth
neon me
neon logout

# Organizations
neon orgs list
neon set-context --org-id <org-id>

# Projects
neon projects list
neon projects get <id>
neon projects create <name>
neon set-context --project-id <project-id>

# Branches
neon branches list
neon branches create <name>
neon branches get <name>
neon branches delete <name>
neon branches switch <name>

# Database
neon sql <branch>
neon connection-string <branch>
neon connection-details <branch>

# Environment
neon env list
neon env set <key> <value>
neon env get <key>

# Monitoring
neon logs <branch>
neon metrics <branch>
```

## Resources

- [Neon CLI Documentation](https://neon.tech/docs/reference/cli)
- [Neon Console](https://console.neon.tech)
- [Neon Discord Community](https://discord.gg/neon)
- [Neon GitHub Repository](https://github.com/neondatabase/neonctl)
