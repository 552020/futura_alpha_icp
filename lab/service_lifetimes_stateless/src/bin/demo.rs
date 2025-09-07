fn main() {
    let mut store = service_lifetimes_stateless::Store::default();
    let svc = service_lifetimes_stateless::StatelessService::default();
    svc.do_it(&mut store);
    println!("n = {}", store.n);
}
