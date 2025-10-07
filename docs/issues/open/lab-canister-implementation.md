# Lab Canister Implementation

## Overview

Create a dedicated laboratory environment with both backend (`lab_backend`) and frontend (`lab_frontend`) canisters for safe experimentation with new features, patterns, and integrations without affecting production code. This will repurpose the existing dead frontend canister.

## Motivation

Currently, we have:

- `src/backend/` - Production backend canister
- `src/canister_factory/` - Production canister factory
- `src/frontend/` - **Dead frontend canister** (unused, can be repurposed)
- `lab/` - Experimental Rust crates (not canisters)

**Problem**: We need a way to experiment with new canister features, test integrations, and prototype functionality in a real ICP environment without risking production stability. We also have a dead frontend canister that's taking up space.

**Solution**:

1. Repurpose the dead `src/frontend/` as `lab_frontend`
2. Create a new `lab_backend` canister (avoiding confusion with main backend)
3. Create a complete lab environment for experimentation

## Technical Proposal

### 1. Project Structure

```
src/
├── backend/           # Production backend canister
├── canister_factory/  # Production canister factory
├── lab_backend/      # NEW: Laboratory backend canister
│   ├── Cargo.toml
│   ├── lab_backend.did
│   └── src/
│       └── lib.rs
└── lab_frontend/     # REPURPOSED: Laboratory frontend canister
    ├── package.json
    ├── vite.config.js
    └── src/
        └── App.jsx
```

### 2. dfx.json Configuration

Replace the dead frontend canister and add the lab backend:

```json
{
  "canisters": {
    "backend": { ... },
    "canister_factory": { ... },
    "internet_identity": { ... },
    "lab_backend": {
      "candid": "src/lab_backend/lab_backend.did",
      "package": "lab_backend",
      "type": "rust",
      "declarations": {
        "output": "src/lab_frontend/src/ic/declarations/lab_backend"
      }
    },
    "lab_frontend": {
      "source": ["src/lab_frontend/dist"],
      "type": "assets"
    }
  }
}
```

### 3. Cargo.toml Updates

Add to workspace `Cargo.toml`:

```toml
[workspace]
members = [
    "src/backend",
    "src/canister_factory",
    "src/lab_backend",  # NEW
    "lab/service_lifetimes_stateless",
    "lab/service_lifetimes_threadlocal",
    "lab/ic_upload_minimal",
]

# Keep lab_backend out of default build for performance
default-members = [
    "src/backend",
    "src/canister_factory"
]
```

### 4. Lab Backend Implementation

**src/lab_backend/Cargo.toml:**

```toml
[package]
name = "lab_backend"
version = "0.1.0"
edition = "2021"

[dependencies]
ic-cdk = "0.12.0"
ic-cdk-macros = "0.8.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**src/lab_backend/src/lib.rs:**

```rust
use ic_cdk::query;
use ic_cdk::update;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ExperimentResult {
    pub success: bool,
    pub data: String,
    pub timestamp: u64,
}

#[query]
fn greet(name: String) -> String {
    format!("Hello from lab_backend canister, {}!", name)
}

#[update]
fn run_experiment(experiment_name: String) -> ExperimentResult {
    ExperimentResult {
        success: true,
        data: format!("Experiment '{}' completed", experiment_name),
        timestamp: ic_cdk::api::time(),
    }
}

#[query]
fn get_status() -> String {
    "Lab backend is ready for experiments".to_string()
}
```

**src/lab_backend/lab_backend.did:**

```candid
service : {
    greet: (text) -> (text);
    run_experiment: (text) -> (record {
        success: bool;
        data: text;
        timestamp: nat64;
    });
    get_status: () -> (text);
}
```

### 5. Lab Frontend Implementation

**src/lab_frontend/src/App.jsx:**

```jsx
import { useState } from "react";
import { lab_backend } from "declarations/lab_backend";

function App() {
  const [greeting, setGreeting] = useState("");
  const [experimentResult, setExperimentResult] = useState("");

  function handleSubmit(event) {
    event.preventDefault();
    const name = event.target.elements.name.value;
    lab_backend.greet(name).then((greeting) => {
      setGreeting(greeting);
    });
    return false;
  }

  function runExperiment() {
    lab_backend.run_experiment("test_experiment").then((result) => {
      setExperimentResult(JSON.stringify(result, null, 2));
    });
  }

  return (
    <main>
      <h1>🧪 Lab Environment</h1>
      <img src="/logo2.svg" alt="DFINITY logo" />
      <br />
      <br />
      <form action="#" onSubmit={handleSubmit}>
        <label htmlFor="name">Enter your name: &nbsp;</label>
        <input id="name" alt="Name" type="text" />
        <button type="submit">Test Greeting</button>
      </form>
      <section id="greeting">{greeting}</section>

      <br />
      <button onClick={runExperiment}>Run Test Experiment</button>
      <pre>{experimentResult}</pre>
    </main>
  );
}

export default App;
```

## Implementation Plan

### Phase 1: Basic Setup

- [ ] Rename `src/frontend/` to `src/lab_frontend/`
- [ ] Create `src/lab_backend/` directory structure
- [ ] Add `Cargo.toml` with minimal dependencies for lab_backend
- [ ] Implement basic `lib.rs` with hello world functionality
- [ ] Create initial `lab_backend.did` file
- [ ] Update workspace `Cargo.toml`
- [ ] Update `dfx.json` configuration (remove old frontend, add lab_backend and lab_frontend)

### Phase 2: Build & Deploy

- [ ] Test local build: `dfx build lab_backend` and `dfx build lab_frontend`
- [ ] Test local deployment: `dfx canister create lab_backend && dfx canister install lab_backend`
- [ ] Test local deployment: `dfx canister create lab_frontend && dfx canister install lab_frontend`
- [ ] Verify lab_frontend can call lab_backend
- [ ] Add to deployment scripts

### Phase 3: Integration

- [ ] Update lab_frontend to use lab_be declarations
- [ ] Create enhanced frontend interface for lab experiments
- [ ] Test full lab environment workflow
- [ ] Document usage patterns

## Usage Examples

### Local Development

```bash
# Build lab canisters
dfx build lab_be
dfx build lab_frontend

# Deploy locally
dfx canister create lab_be
dfx canister install lab_be
dfx canister create lab_frontend
dfx canister install lab_frontend

# Test via dfx
dfx canister call lab_be greet "World"
dfx canister call lab_be get_status

# Access lab frontend
dfx canister id lab_frontend
# Open browser to: http://localhost:4943/?canisterId=<lab_frontend_id>
```

### Frontend Integration

```typescript
import { lab_be } from "declarations/lab_be";

// Call lab_be canister from lab_frontend
const result = await lab_be.greet("Developer");
const status = await lab_be.get_status();
const experiment = await lab_be.run_experiment("my_test");
```

## Benefits

1. **Safe Experimentation**: Test new features without affecting production
2. **Rapid Prototyping**: Quick iteration on new ideas with full frontend/backend environment
3. **Integration Testing**: Test canister-to-canister communication
4. **Performance Testing**: Benchmark new approaches
5. **Learning Platform**: Safe environment for team members to learn ICP patterns
6. **Code Reuse**: Repurposes existing dead frontend canister
7. **Better DX**: Clear naming (`lab_be` vs `backend`) avoids confusion

## Considerations

### Security

- Lab canister should have minimal permissions
- No access to production data or critical functions
- Consider separate identity/authentication for lab environment

### Resource Management

- Lab canisters excluded from default build for performance
- Can be deployed independently of production canisters
- Consider separate cycles management for lab environment

### Maintenance

- Regular cleanup of experimental code
- Documentation of successful experiments for potential production migration
- Clear separation between lab and production code

## Future Enhancements

1. **Experiment Categories**: Organize experiments by type (auth, storage, performance, etc.)
2. **A/B Testing Framework**: Built-in support for comparing approaches
3. **Metrics Collection**: Automatic performance and resource usage tracking
4. **Experiment Templates**: Common patterns for different types of experiments
5. **Integration with CI/CD**: Automated testing of lab experiments

## Related Issues

- Consider how this relates to existing `lab/` crates (they serve different purposes)
- Integration with testing strategy
- Potential migration path from lab to production

## Acceptance Criteria

- [ ] Lab backend (`lab_be`) builds successfully
- [ ] Lab frontend (`lab_frontend`) builds successfully
- [ ] Both canisters deploy locally without errors
- [ ] Basic functionality (greet, status, experiments) works
- [ ] Lab frontend can call lab backend
- [ ] Dead frontend canister is properly repurposed
- [ ] Documentation is complete
- [ ] No impact on existing production canisters
- [ ] Team can use lab environment for experiments

## Priority

**Medium** - This is a valuable infrastructure addition that will improve development velocity and safety, but not blocking current features.

## Estimated Effort

**3-5 hours** - Configuration, repurposing existing frontend, and basic setup. Slightly more than original estimate due to frontend integration.
