// LLM Chatbot Module - Modern Rust module structure
// This module contains the complete ic-llm functionality copied from the official repository

mod chat;
mod tool;

// Re-export the main LLM functionality
pub use chat::{AssistantMessage, ChatMessage, FunctionCall, Response, ToolCall};
pub use tool::{Function, ParameterType, Parameters, Property, Tool};

// Import the internal modules
use chat::ChatBuilder;
use std::fmt;

// The principal of the LLM canister
const LLM_CANISTER: &str = "w36hm-eqaaa-aaaal-qr76a-cai";

/// Supported LLM models
#[derive(Debug)]
pub enum Model {
    Llama3_1_8B,
    Qwen3_32B,
    Llama4Scout,
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Model::Llama3_1_8B => "llama3.1:8b",
            Model::Qwen3_32B => "qwen3:32b",
            Model::Llama4Scout => "llama4-scout",
        };
        write!(f, "{}", text)
    }
}

/// Sends a single message to a model
pub async fn prompt<P: ToString>(model: Model, prompt_str: P) -> String {
    let response = ChatBuilder::new(model)
        .with_messages(vec![ChatMessage::User {
            content: prompt_str.to_string(),
        }])
        .send()
        .await;

    response.message.content.unwrap_or_default()
}

/// Creates a new ChatBuilder with the specified model
pub fn chat(model: Model) -> ChatBuilder {
    ChatBuilder::new(model)
}

/// Creates a new ToolBuilder with the specified name
pub fn tool<S: Into<String>>(name: S) -> tool::ToolBuilder {
    tool::ToolBuilder::new(name)
}

/// Creates a new ParameterBuilder with the specified name and type
pub fn parameter<S: Into<String>>(name: S, type_: ParameterType) -> tool::ParameterBuilder {
    tool::ParameterBuilder::new(name, type_)
}

// ============================================================================
// CANISTER INTERFACE FUNCTIONS
// ============================================================================

use ic_cdk::update;

/// Canister interface: Send a single prompt to the LLM
#[update]
pub async fn prompt_canister(prompt_str: String) -> String {
    prompt(Model::Llama3_1_8B, prompt_str).await
}

/// Canister interface: Chat with the LLM using message history
#[update]
pub async fn chat_canister(messages: Vec<ChatMessage>) -> String {
    let response = chat(Model::Llama3_1_8B)
        .with_messages(messages)
        .send()
        .await;

    response.message.content.unwrap_or_default()
}
