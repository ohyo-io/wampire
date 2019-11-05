use argparse::{ArgumentParser, Store, StoreTrue};
use env_logger;

use wampire::router::Router;

fn main() {
    env_logger::init();

    let mut verbose = false;
    let mut port = "8090".to_string();
    let mut realm = "wampire_realm".to_string();
    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Options");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");
        ap.refer(&mut port)
            .add_option(&["-P", "--port"], Store, "Listening port");
        ap.refer(&mut realm)
            .add_option(&["-R", "--realm"], Store, "Handling realm");
        ap.parse_args_or_exit();
    }

    if verbose {
        println!("Handling realm [{}] on {} port", realm, port);
    }

    let mut router = Router::new();
    router.add_realm(realm.as_str());

    let addr = format!("127.0.0.1:{}", port);
    let child = router.listen(addr.as_str());
    child.join().unwrap();
}
