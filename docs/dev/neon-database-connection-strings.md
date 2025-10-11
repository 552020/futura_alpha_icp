# Neon Database Connection Strings Guide

This guide explains how to retrieve branch-specific Neon database connection strings for your Futura project, including PR preview databases created by Vercel.

## Overview

When Vercel creates preview deployments for pull requests, it automatically creates corresponding Neon database branches. You can retrieve the connection strings for these branches using the Neon CLI to:

- Debug database issues in PR environments
- Connect directly to PR databases with `psql` or other tools
- Run migrations or queries against specific PR databases
- Inspect data in preview environments

## Prerequisites

- Neon CLI (`neonctl`) installed
- Access to the Vercel organization's Neon projects
- Authentication with Neon CLI

## Quick Setup

### 1. Verify Installation and Authentication

```bash
# Check if neonctl is installed
which neonctl

# Check authentication status
neonctl me
```

### 2. Set Context

Set the context to the Vercel organization and your project:

```bash
neonctl set-context --org-id org-small-water-18243211 --project-id dark-river-75738037
```

## Retrieving Connection Strings

### List All Available Branches

```bash
neonctl branches list
```

This will show all branches including:

- `main` - Production database
- `preview/pr-XX-*` - PR preview databases
- `dev/552020` - Development database
- `staging` - Staging database

### Get Connection String for Specific Branch

```bash
# Main branch
neonctl connection-string br-crimson-wildflower-a21a09u3

# PR branch (example)
neonctl connection-string br-nameless-firefly-a2ldpggc
```

## Current Project Branches

| Branch ID                        | Name                                                                 | Type        | Status   |
| -------------------------------- | -------------------------------------------------------------------- | ----------- | -------- |
| `br-crimson-wildflower-a21a09u3` | main                                                                 | Production  | ready    |
| `br-nameless-firefly-a2ldpggc`   | preview/pr-51-552020/icp-413-wire-icp-memory-upload-frontend-backend | PR #51      | ready    |
| `br-round-hill-a2oqy10v`         | preview/pr-54-lmangallon/icp-545-db-october-improvements             | PR #54      | ready    |
| `br-sparkling-morning-a2dfg8ar`  | dev/552020                                                           | Development | ready    |
| `br-delicate-bonus-a2063uec`     | staging                                                              | Staging     | archived |

## Usage Examples

### Connect with psql

```bash
# Get connection string and connect
neonctl connection-string br-nameless-firefly-a2ldpggc | xargs psql
```

### Use in Environment Variables

```bash
# Export for current session
export DATABASE_URL=$(neonctl connection-string br-nameless-firefly-a2ldpggc)

# Use in scripts
DATABASE_URL=$(neonctl connection-string br-nameless-firefly-a2ldpggc) npm run db:migrate
```

### Find Branch by PR Number

```bash
# List branches and filter by PR number
neonctl branches list | grep "pr-51"

# Get the branch ID and connection string
BRANCH_ID=$(neonctl branches list --output json | jq -r '.branches[] | select(.name | contains("pr-51")) | .id')
neonctl connection-string $BRANCH_ID
```

## Project Information

- **Organization**: Vercel: 552020's projects (`org-small-water-18243211`)
- **Project**: futura_alpha_db (`dark-river-75738037`)
- **Region**: aws-eu-central-1
- **Created**: 2025-02-12T20:14:17Z

## Troubleshooting

### Context Reset

If you get "No projects found" error, reset the context:

```bash
neonctl set-context --org-id org-small-water-18243211 --project-id dark-river-75738037
```

### Authentication Issues

If authentication fails:

```bash
neonctl auth
```

### List Organizations

To see available organizations:

```bash
neonctl orgs list
```

## Automation Scripts

### Get Connection String by PR Number

Create a script `get-pr-db.sh`:

```bash
#!/bin/bash
PR_NUMBER=$1

if [ -z "$PR_NUMBER" ]; then
    echo "Usage: $0 <pr-number>"
    exit 1
fi

# Set context
neonctl set-context --org-id org-small-water-18243211 --project-id dark-river-75738037

# Find branch by PR number
BRANCH_ID=$(neonctl branches list --output json | jq -r ".branches[] | select(.name | contains(\"pr-$PR_NUMBER\")) | .id")

if [ "$BRANCH_ID" = "null" ] || [ -z "$BRANCH_ID" ]; then
    echo "No database found for PR #$PR_NUMBER"
    exit 1
fi

# Get connection string
neonctl connection-string $BRANCH_ID
```

Make it executable and use:

```bash
chmod +x get-pr-db.sh
./get-pr-db.sh 51
```

### Connect to PR Database

Create a script `connect-pr-db.sh`:

```bash
#!/bin/bash
PR_NUMBER=$1

if [ -z "$PR_NUMBER" ]; then
    echo "Usage: $0 <pr-number>"
    exit 1
fi

# Get connection string and connect
CONNECTION_STRING=$(./get-pr-db.sh $PR_NUMBER)
if [ $? -eq 0 ]; then
    psql "$CONNECTION_STRING"
else
    echo "Failed to get connection string for PR #$PR_NUMBER"
    exit 1
fi
```

## Notes

- PR branches are automatically created by Vercel when preview deployments are triggered
- Each PR gets its own isolated database branch
- Connection strings include SSL requirements (`sslmode=require`)
- Branches may be automatically cleaned up after PRs are closed
- The main branch (`br-crimson-wildflower-a21a09u3`) is the default production database

## Authentication & Sharing Access

### How Neon CLI Authentication Works

The Neon CLI uses OAuth2 tokens stored in `~/.config/neonctl/credentials.json`. These tokens include:

- **Access Token**: Short-lived token for API calls
- **Refresh Token**: Long-lived token to get new access tokens
- **ID Token**: Contains user information
- **Expiration**: Tokens expire and are automatically refreshed

### Sharing Access with Team Members

#### Option 1: Individual Authentication (Recommended)

Each team member should authenticate with their own Neon account:

```bash
# On the new computer
neonctl auth
# Follow the browser authentication flow
```

Then they can access the same projects if they're added to the Vercel organization.

#### Option 2: Share Credentials File (Not Recommended)

⚠️ **Security Risk**: This gives full access to your Neon account.

If you must share access temporarily:

```bash
# Copy credentials to another computer
scp ~/.config/neonctl/credentials.json user@other-computer:~/.config/neonctl/
```

**Risks:**

- Full access to your Neon account
- Can create/delete projects and databases
- Tokens can be refreshed and used indefinitely
- No audit trail of who did what

#### Option 3: API Key (Most Secure)

Generate a project-specific API key for limited access:

1. Go to [Neon Console](https://console.neon.tech/)
2. Navigate to your project
3. Go to Settings → API Keys
4. Create a new API key with limited permissions
5. Use it with the CLI:

```bash
neonctl --api-key YOUR_API_KEY projects list
```

### What You Need on Another Computer

#### For Full Access (Your Account):

1. **neonctl installed**: `npm install -g neonctl` or download binary
2. **Your credentials file**: `~/.config/neonctl/credentials.json`
3. **Same context**: Organization and project IDs

#### For Limited Access (API Key):

1. **neonctl installed**
2. **API key** with appropriate permissions
3. **Project ID**: `dark-river-75738037`

### Security Best Practices

1. **Use individual accounts** when possible
2. **Rotate API keys** regularly
3. **Limit API key permissions** to only what's needed
4. **Never commit credentials** to version control
5. **Use environment variables** for API keys in scripts

### Revoking Access

If you need to revoke access:

1. **Change your password** on Neon console
2. **Delete API keys** you've shared
3. **Remove team members** from Vercel organization
4. **Regenerate tokens** by running `neonctl auth` again

## Related Documentation

- [Neon CLI Documentation](https://neon.com/docs/reference/neon-cli)
- [Vercel Preview Deployments](https://vercel.com/docs/concepts/deployments/preview-deployments)
- [Neon Branching](https://neon.com/docs/concepts/branching)
- [Neon API Keys](https://neon.com/docs/management/api-keys)
