use std::cell::RefCell;

#[derive(Default, Debug, PartialEq)]
pub struct Store { pub n: u32 }
impl Store { pub fn inc(&mut self) { self.n += 1; } }

thread_local! {
    static GLOBAL_STORE: RefCell<Store> = RefCell::new(Store::default());
}

#[derive(Default)]
pub struct GlobalService;

impl GlobalService {
    pub fn do_it(&self) {
        GLOBAL_STORE.with(|s| s.borrow_mut().inc());
    }
    pub fn read(&self) -> u32 {
        GLOBAL_STORE.with(|s| s.borrow().n)
    }
    /// Reset for test isolation
    pub fn reset(&self) {
        GLOBAL_STORE.with(|s| *s.borrow_mut() = Store::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increments_global() {
        let svc = GlobalService::default();
        svc.reset();
        svc.do_it();
        assert_eq!(svc.read(), 1);
    }
}