use crate::http::core_types::Clock;
use ic_cdk::api::time;

pub struct CanisterClock;

impl Clock for CanisterClock {
    fn now_ns(&self) -> u64 {
        time()
    }
}
