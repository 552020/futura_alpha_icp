# Sources and References

This document cites all the relevant documentation and resources referenced in the article "From Next.js Image Error to ICP HTTP Module: A Journey of Decentralized Asset Serving".

## Internal Documentation

### Implementation Documentation

- [HTTP Request Implementation Final](../issues/open/serving-http/http_request_implementation_final.md)
- [HTTP Request Implementation](../issues/open/serving-http/http_request_implementation.md)
- [Phase 1 Implementation TODOs](../issues/open/serving-http/phase1-implementation-todos.md)
- [Architecture Clarification Answers](../issues/open/serving-http/architecture-clarification-answers.md)

### Issue Tracking and Analysis

- [Next.js Image Component Optimization Analysis](../../../src/nextjs/docs/issues/nextjs-image-component-optimization-analysis.md) - **Original "output: 'export'" error that started the journey**
- [ICP Image Configuration Issue - Next.js Image Component](../issues/open/serving-http/icp-image-configuration-nextjs.md) - **ICP-specific image serving challenges**
- [HTTP Module Compilation WASM Compatibility Issues](../issues/open/serving-http/http-module-compilation-wasm-compatibility-issues.md)
- [Phase 1 Implementation Blockers](../issues/open/serving-http/phase1-implementation-blockers.md)
- [Implementation Blockers and Solutions](../issues/open/serving-http/implementation-blockers-and-solutions.md)
- [Tech Lead 9 Point Feedback](../issues/open/serving-http/tech-lead-9-point-feedback.md)

### Related WASM Compatibility Issues

- [UUID v7 Deployment WASM Compatibility Issues](../issues/open/uuid-memories/uuid-v7-deployment-wasm-compatibility-issues.md)

### Architecture Analysis

- [CDK-RS Official API Analysis](../issues/open/serving-http/cdk-rs-official-api-analysis.md)
- [Domain Integration Analysis](../issues/open/serving-http/domain-integration-analysis.md)

## Code Implementation

### Core HTTP Module

- `src/backend/src/http/core/types.rs` - Core types and traits
- `src/backend/src/http/adapters/acl.rs` - ACL integration adapter
- `src/backend/src/http/adapters/asset_store.rs` - Asset store bridge
- `src/backend/src/http/adapters/secret_store.rs` - Secret management
- `src/backend/src/lib.rs` - Main canister integration

### Configuration

- `src/backend/Cargo.toml` - Dependencies and WASM compatibility notes

### Testing

- `tests/backend/http/test_http_basic.sh` - Basic HTTP functionality tests
- `tests/backend/http/run_http_tests.sh` - Test runner

## External Resources

### ICP Documentation

- [ICP HTTP Request Documentation](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request)
- [ICP Execution Errors](https://internetcomputer.org/docs/current/references/execution-errors#calling-a-system-api-from-the-wrong-mode)

### Rust and WASM

- [Getrandom WebAssembly Support](https://docs.rs/getrandom/#webassembly-support)
- [Rust WASM Target Documentation](https://doc.rust-lang.org/nightly/rustc/platform-support/wasm32-unknown-unknown.html)

### Next.js

- [Next.js Image Component](https://nextjs.org/docs/api-reference/next/image)
- [Next.js Output Export](https://nextjs.org/docs/advanced-features/static-html-export)

## Project Repository

- [Futura Alpha ICP Repository](https://github.com/futura-icp/futura_alpha_icp)

---

_This sources document provides comprehensive references for all technical details, implementation decisions, and external resources mentioned in the main article._
