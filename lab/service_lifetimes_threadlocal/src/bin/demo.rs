fn main() {
    let svc = service_lifetimes_threadlocal::GlobalService::default();
    svc.reset();
    svc.do_it();
    println!("n = {}", svc.read());
}
