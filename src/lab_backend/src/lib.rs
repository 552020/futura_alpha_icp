// Module declarations
mod greet;
mod llm_chatbot;

// Re-export functions for the canister interface
pub use greet::{
    compare_approaches, get_status_robust, get_status_simple, greet_robust, greet_simple, health,
    run_experiment_robust, run_experiment_simple,
};
pub use llm_chatbot::{chat_canister, prompt_canister, ChatMessage};

// Re-export types for the canister interface
pub use greet::{ExperimentData, ExperimentResult};

// Export the interface for the smart contract.
ic_cdk::export_candid!();
