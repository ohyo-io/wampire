extern crate wamp;

use wamp::router::Router;
extern crate env_logger;
#[macro_use]
extern crate log;

fn main() {
    env_logger::init().unwrap();
    let mut router = Router::new();
    router.add_realm("kitchen_realm");
    info!("Router listening");
    let child = router.listen("127.0.0.1:8090");
    child.join().unwrap();
}
