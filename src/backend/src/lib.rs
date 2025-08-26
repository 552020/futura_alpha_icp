use candid::Principal;

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[ic_cdk::query]
fn whoami() -> Principal {
    ic_cdk::api::msg_caller()
}

// Export the interface for the smart contract.
ic_cdk::export_candid!();
