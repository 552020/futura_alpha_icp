use ic_cdk::api::time;
use crate::http::core_types::Clock;

pub struct CanisterClock;

impl Clock for CanisterClock {
    fn now_ns(&self) -> u64 { 
        time() 
    }
}
