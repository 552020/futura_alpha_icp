# Documentation

This directory contains project documentation organized by type and status.

## Structure

```
docs/
├── issues/
│   ├── open/          # Active issues and feature requests
│   └── done/          # Resolved issues and completed features
└── README.md          # This file
```

## Issues

### Open Issues (`issues/open/`)

- **BACKEND_FUNCTIONS_NEEDED.md** - Missing backend functions for personal canister management
- **rename_binding_to_neon_functions.md** - Rename outdated "bind to neon" function names
- **memory-api-refactoring-ping-to-get-functions.md** - Replace `ping` with `get_memory` and `get_memory_with_assets`
- **rename-capsules-bind-neon-to-storage-edges.md** - Rename `capsules_bind_neon` to reflect new database storage edges architecture

### Resolved Issues (`issues/done/`)

- **memory_binding_and_blob_store_issues.md** - Backend test issues (all resolved - 100% test success rate)

## Contributing

When creating new issues:

1. Place them in `issues/open/`
2. Use descriptive filenames
3. Include status, priority, and creation date
4. Move to `issues/done/` when resolved

## Naming Convention

- Use lowercase with underscores for filenames
- Include issue type in filename when relevant
- Use descriptive names that explain the content
