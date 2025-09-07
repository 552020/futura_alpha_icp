#[derive(Default, Debug, PartialEq)]
pub struct Store {
    pub n: u32,
}
impl Store {
    pub fn inc(&mut self) {
        self.n += 1;
    }
}

#[derive(Default)]
pub struct StatelessService;

impl StatelessService {
    pub fn do_it(&self, store: &mut Store) {
        store.inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increments_store() {
        let mut store = Store::default();
        let svc = StatelessService::default();

        svc.do_it(&mut store);
        assert_eq!(store.n, 1);
    }

    #[test]
    fn can_return_both_from_helper() {
        fn helper() -> (StatelessService, Store) {
            (StatelessService::default(), Store::default())
        }
        let (svc, mut store) = helper();
        svc.do_it(&mut store);
        assert_eq!(store.n, 1);
    }
}
