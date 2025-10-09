# Lab Backend Documentation

This directory contains reference documentation for the LLM functionality implemented in the lab_backend canister.

## Structure

### `ic-llm/`

Reference files from the official DFinity LLM repository:

- **`README.md`** - Original documentation for the ic-llm crate
- **`Cargo.toml`** - Original dependencies (ic-cdk 0.17.1)
- **`LICENSE`** - Original Apache 2.0 license

### `icp-ninja/`

Reference files from the ICP Ninja LLM chatbot example:

- **`README.md`** - Original ICP Ninja example documentation
- **`Cargo.toml`** - Original dependencies (ic-cdk 0.17.1, ic-llm 1.1.0)

## Implementation Notes

The lab_backend canister implements LLM functionality by:

1. **Copying the source code** from the ic-llm repository into `src/llm_chatbot/`
2. **Adapting it** to work with ic-cdk 0.18 (no external dependencies)
3. **Providing canister interfaces** (`prompt_canister`, `chat_canister`)

## Key Benefits

- ✅ **No dependency conflicts** - Self-contained implementation
- ✅ **Full control** - Can modify and extend as needed
- ✅ **ic-cdk 0.18 compatible** - Works with latest version
- ✅ **Reference preserved** - Original files available for comparison

## Usage

The LLM functionality is available through:

- `prompt_canister(prompt: String) -> String`
- `chat_canister(messages: Vec<ChatMessage>) -> String`

See `src/llm_chatbot.rs` for the implementation details.
