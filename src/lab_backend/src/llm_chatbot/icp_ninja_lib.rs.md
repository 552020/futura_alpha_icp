# ICP Ninja Example - Original Implementation

**Source**: ICP Ninja LLM Chatbot example  
**Purpose**: Simple example using external ic-llm crate  
**Dependencies**: `ic-llm = "1.1.0"`, `ic-cdk = "0.17.1"`

## Original Source Code

```rust
use ic_cdk::update;
use ic_llm::{ChatMessage, Model};

// ============================================================================
// LLM MODULE - AI/LLM functionality for experiments
// ============================================================================

#[update]
pub async fn prompt(prompt_str: String) -> String {
    ic_llm::prompt(Model::Llama3_1_8B, prompt_str).await
}

#[update]
pub async fn chat(messages: Vec<ChatMessage>) -> String {
    let response = ic_llm::chat(Model::Llama3_1_8B)
        .with_messages(messages)
        .send()
        .await;

    response.message.content.unwrap_or_default()
}
```

## Notes

- This was our original implementation that used the external `ic-llm` crate
- Simple wrapper around the ic-llm API
- Caused dependency conflicts with ic-cdk 0.18
- Replaced by our new self-contained implementation in `llm_chatbot.rs`
