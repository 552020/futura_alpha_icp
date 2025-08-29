# Canister Factory Pattern in Juno

## Overview

Juno implements a sophisticated canister factory pattern that enables the creation and management of different types of canisters (called "segments") through a centralized system. The architecture follows a space-themed naming convention with three main components:

- **Mission Control**: The central factory that manages and creates other canisters
- **Satellites**: Canisters that provide data storage and hosting capabilities
- **Orbiters**: Canisters that provide analytics and monitoring capabilities

## Most Relevant Files and Folders

### Core Factory Components

#### Mission Control (Factory Hub)

- **`src/mission_control/src/api/satellites.rs`** - Satellite creation API endpoints
- **`src/mission_control/src/api/orbiters.rs`** - Orbiter creation API endpoints
- **`src/mission_control/src/segments/satellite.rs`** - Satellite creation logic
- **`src/mission_control/src/segments/orbiter.rs`** - Orbiter creation logic
- **`src/mission_control/src/segments/canister.rs`** - Generic canister creation utilities
- **`src/mission_control/mission_control.did`** - Mission Control interface definition

#### Console (Canister Creation Service)

- **`src/console/console.did`** - Console interface with `create_satellite` and `create_orbiter` methods
- **`src/console/src/`** - Console implementation (actual canister creation logic)

#### Created Canister Types

- **`src/satellite/satellite.did`** - Satellite canister interface (data storage & hosting)
- **`src/orbiter/orbiter.did`** - Orbiter canister interface (analytics & monitoring)

### Supporting Infrastructure

#### Shared Libraries

- **`src/libs/shared/`** - Common types and utilities used across all components
- **`src/libs/satellite/`** - Satellite-specific shared code
- **`src/libs/utils/`** - Utility functions

#### Configuration and Management

- **`src/mission_control/src/segments/store.rs`** - State management for created canisters
- **`src/mission_control/src/controllers/`** - Controller management logic
- **`src/mission_control/src/monitoring/`** - Cycles monitoring and funding logic

### Key Entry Points for Understanding the Pattern

1. **Start with**: `src/mission_control/src/api/satellites.rs` - Shows how users interact with the factory
2. **Follow the flow**: `src/mission_control/src/segments/satellite.rs` - Shows the creation process
3. **See the interface**: `src/console/console.did` - Shows what the factory calls to create canisters
4. **Understand the types**: `src/mission_control/mission_control.did` - Shows the data structures

## Architecture Components

### 1. Mission Control (`src/mission_control/`)

Mission Control serves as the central factory and management hub for all other canisters. It implements the core factory pattern with the following key responsibilities:

#### Key Functions:

- `create_satellite(name: String)` - Creates a new satellite canister
- `create_satellite_with_config(config: CreateCanisterConfig)` - Creates satellite with custom configuration
- `create_orbiter(name: Option<String>)` - Creates a new orbiter canister
- `create_orbiter_with_config(config: CreateCanisterConfig)` - Creates orbiter with custom configuration

#### Factory Pattern Implementation:

```rust
// From src/mission_control/src/api/satellites.rs
#[update(guard = "caller_is_user_or_admin_controller")]
async fn create_satellite(name: String) -> Satellite {
    create_satellite_console(&name)
        .await
        .unwrap_or_else(|e| trap(&e))
}

#[update(guard = "caller_is_user_or_admin_controller")]
async fn create_satellite_with_config(config: CreateCanisterConfig) -> Satellite {
    create_satellite_with_config_console(&config)
        .await
        .unwrap_or_else(|e| trap(&e))
}
```

### 2. Console (`src/console/`)

The Console acts as the actual canister creation service that Mission Control calls to instantiate new canisters:

#### Key Functions:

- `create_satellite(args: CreateCanisterArgs) -> principal` - Creates satellite canister
- `create_orbiter(args: CreateCanisterArgs) -> principal` - Creates orbiter canister
- `get_create_satellite_fee(args: GetCreateCanisterFeeArgs) -> opt Tokens` - Gets creation fee
- `get_create_orbiter_fee(args: GetCreateCanisterFeeArgs) -> opt Tokens` - Gets creation fee

### 3. Satellite (`src/satellite/`)

Satellites are canisters that provide:

- Data storage (documents and assets)
- File hosting capabilities
- Authentication management
- Custom domain support

#### Key Features:

- Document storage and retrieval
- Asset management and hosting
- Authentication configuration
- Custom domain management
- Proposal system for content management

### 4. Orbiter (`src/orbiter/`)

Orbiters are canisters that provide:

- Analytics and monitoring
- Page view tracking
- Performance metrics
- Event tracking
- Web vitals monitoring

#### Key Features:

- Page view analytics
- Performance metrics collection
- Event tracking
- Browser and device analytics
- Web vitals monitoring

## Factory Pattern Flow

### 1. Canister Creation Process

The factory pattern follows this flow:

```
User Request → Mission Control → Console → Canister Creation
```

1. **User calls Mission Control**: User requests creation of satellite/orbiter
2. **Mission Control validates**: Checks permissions and gets creation fee
3. **Payment processing**: If fee required, transfers payment to Console
4. **Console creates canister**: Actually instantiates the new canister
5. **Registration**: Mission Control registers the new canister in its state

### 2. Implementation Details

#### Fee Management:

```rust
// From src/mission_control/src/segments/canister.rs
pub async fn create_canister<F, Fut, T>(
    fee_method: &str,
    create_and_save: F,
    config: &CreateCanisterConfig,
) -> Result<T, String>
```

The factory pattern includes sophisticated fee management:

- Queries Console for creation fees
- Handles free vs paid canister creation
- Manages ICP token transfers for payments

#### Canister Configuration:

```rust
type CreateCanisterConfig = record {
    subnet_id : opt principal;
    name : opt text;
};
```

Supports custom configuration including:

- Subnet placement
- Canister naming
- Custom initialization parameters

### 3. State Management

Mission Control maintains state for all created canisters:

```rust
// From mission_control.did
type Satellite = record {
    updated_at : nat64;
    metadata : vec record { text; text };
    created_at : nat64;
    satellite_id : principal;
    settings : opt Settings;
};

type Orbiter = record {
    updated_at : nat64;
    orbiter_id : principal;
    metadata : vec record { text; text };
    created_at : nat64;
    settings : opt Settings;
};
```

## Key Design Patterns

### 1. Factory Method Pattern

- Mission Control acts as the factory
- Console provides the actual creation logic
- Different creation methods for different canister types

### 2. Registry Pattern

- Mission Control maintains a registry of all created canisters
- Provides listing and management capabilities
- Enables centralized control and monitoring

### 3. Controller Pattern

- Mission Control can set controllers on created canisters
- Supports different controller scopes (Write, Admin, Submit)
- Enables fine-grained access control

### 4. Monitoring Pattern

- Built-in cycles monitoring for all canisters
- Automatic funding when cycles are low
- Configurable monitoring strategies

## Security Features

### 1. Access Control

- Guard functions ensure only authorized users can create canisters
- Controller management for fine-grained permissions
- Expiration dates for temporary access

### 2. Payment Security

- Fee validation before canister creation
- Secure ICP token transfers
- Transaction verification

### 3. Canister Validation

- Verification that created canisters are of correct type
- Function signature validation
- Controller relationship verification

## Usage Examples

### Creating a Satellite

```rust
// User calls Mission Control
let satellite = mission_control.create_satellite("my-satellite".to_string()).await;

// Mission Control:
// 1. Gets creation fee from Console
// 2. Transfers payment if required
// 3. Calls Console to create canister
// 4. Registers satellite in state
// 5. Returns satellite info
```

### Creating an Orbiter

```rust
// User calls Mission Control
let orbiter = mission_control.create_orbiter(Some("my-orbiter".to_string())).await;

// Similar flow to satellite creation
```

### Managing Controllers

```rust
// Set controllers on multiple satellites
mission_control.set_satellites_controllers(
    vec![satellite_id1, satellite_id2],
    vec![controller_id1, controller_id2],
    SetController {
        metadata: vec![("purpose", "analytics")],
        scope: ControllerScope::Write,
        expires_at: Some(expiration_timestamp),
    }
).await;
```

## Benefits of This Pattern

1. **Centralized Management**: All canister creation goes through Mission Control
2. **Consistent Configuration**: Standardized creation process for all canisters
3. **Resource Management**: Built-in cycles monitoring and funding
4. **Security**: Centralized access control and validation
5. **Scalability**: Easy to add new canister types
6. **Monitoring**: Built-in analytics and monitoring capabilities

## Conclusion

Juno's canister factory pattern provides a robust, secure, and scalable way to create and manage different types of canisters. The space-themed architecture makes it intuitive to understand, while the underlying patterns ensure proper resource management, security, and monitoring. This pattern could serve as an excellent reference for implementing similar factory patterns in other Internet Computer projects.
