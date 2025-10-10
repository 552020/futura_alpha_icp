# Access Control System - JavaScript Demo

This is a simplified JavaScript version of the Rust access control system we're building for the ICP capsule system.

## What This Demonstrates

- **Centralized Access Control**: One place to manage all permissions
- **Resource Keys**: How to identify any resource (Memory, Gallery, etc.)
- **Permission Evaluation**: How to check if someone can access something
- **Bitflags**: How to use bitwise operations for permissions

## Files

- `index.js` - Main demo with examples
- `permissions.js` - Permission bitflags system
- `access-index.js` - Centralized access storage
- `demo.js` - Interactive examples

## Run It

```bash
node index.js
```

## The Problem We're Solving

Instead of having access control scattered everywhere, we want:
- One place to check permissions
- One place to manage who has access to what
- Support for owners, controllers, groups, public access, magic links
