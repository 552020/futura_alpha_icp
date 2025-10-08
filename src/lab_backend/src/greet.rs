use candid::CandidType;
use ic_cdk::query;
use ic_cdk::update;
use serde::{Deserialize, Serialize};

// ============================================================================
// SIMPLE APPROACH - Direct returns, minimal error handling
// ============================================================================

#[derive(Serialize, Deserialize, CandidType)]
pub struct ExperimentResult {
    pub success: bool,
    pub data: String,
    pub timestamp: u64,
}

#[query]
pub fn greet_simple(name: String) -> String {
    format!("Hello from lab_backend canister, {}!", name)
}

#[update]
pub fn run_experiment_simple(experiment_name: String) -> ExperimentResult {
    ExperimentResult {
        success: true,
        data: format!("Experiment '{}' completed", experiment_name),
        timestamp: ic_cdk::api::time(),
    }
}

#[query]
pub fn get_status_simple() -> String {
    "Lab backend is ready for experiments".to_string()
}

// ============================================================================
// ROBUST APPROACH - Simple error handling with Result<T, String>
// ============================================================================

#[derive(Serialize, Deserialize, CandidType)]
pub struct ExperimentData {
    pub data: String,
    pub timestamp: u64,
}

#[query]
pub fn greet_robust(name: String) -> Result<String, String> {
    if name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if name.len() > 100 {
        return Err("Name too long (max 100 chars)".to_string());
    }

    Ok(format!("Hello from lab_backend canister, {}!", name))
}

#[update]
pub fn run_experiment_robust(experiment_name: String) -> Result<ExperimentData, String> {
    if experiment_name.trim().is_empty() {
        return Err("Experiment name cannot be empty".to_string());
    }

    if experiment_name.len() > 50 {
        return Err("Experiment name too long (max 50 chars)".to_string());
    }

    // Simulate potential failure for demonstration
    if experiment_name == "fail" {
        return Err("Simulated experiment failure".to_string());
    }

    if experiment_name == "unauthorized" {
        return Err("Unauthorized access".to_string());
    }

    Ok(ExperimentData {
        data: format!("Experiment '{}' completed successfully", experiment_name),
        timestamp: ic_cdk::api::time(),
    })
}

#[query]
pub fn get_status_robust() -> Result<String, String> {
    Ok("Lab backend is ready for experiments with simple error handling".to_string())
}

// ============================================================================
// COMMON FUNCTIONS
// ============================================================================

#[query]
pub fn health() -> String {
    "ok".to_string()
}

#[query]
pub fn compare_approaches() -> String {
    r#"
    🧪 Lab Backend - Error Handling Approaches Comparison
    
    SIMPLE APPROACH (direct returns):
    - greet_simple(name) -> String
    - run_experiment_simple(name) -> ExperimentResult
    - get_status_simple() -> String
    
    ROBUST APPROACH (simple error handling):
    - greet_robust(name) -> Result<String, String>
    - run_experiment_robust(name) -> Result<ExperimentData, String>
    - get_status_robust() -> Result<String, String>
    
    Both approaches have their place:
    - Simple: Quick prototyping, internal functions, guaranteed success cases
    - Robust: User-facing APIs, external integrations, input validation
    
    Try both versions to see the difference!
    "#
    .to_string()
}
