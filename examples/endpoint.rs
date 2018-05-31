extern crate wampire;
extern crate eventual;
#[macro_use]
extern crate log;
extern crate env_logger;

use wampire::client::Connection;
use wampire::{URI, Value, Dict, List, CallResult, ArgList};
use std::io;
use eventual::Async;

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn addition_callback(args: List, _kwargs: Dict) -> CallResult<(Option<List>, Option<Dict>)> {
    info!("Performing addition");
    try!(args.verify_len(2));
    let a = try!(args.get_int(0)).unwrap();
    let b = try!(args.get_int(1)).unwrap();
    Ok((Some(vec![Value::Integer(a + b)]), None))
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn multiplication_callback(args: List, _kwargs: Dict) -> CallResult<(Option<List>, Option<Dict>)> {
    info!("Performing multiplication");
    try!(args.verify_len(2));
    let a = try!(args.get_int(0)).unwrap();
    let b = try!(args.get_int(1)).unwrap();
    Ok((Some(vec![Value::Integer(a * b)]), None))
}

fn echo_callback(args: List, kwargs: Dict) -> CallResult<(Option<List>, Option<Dict>)> {
    info!("Performing echo");
    Ok((Some(args), Some(kwargs)))
}

fn main() {
    env_logger::init().unwrap();
    let connection = Connection::new("ws://127.0.0.1:8090/ws", "realm1");
    info!("Connecting");
    let mut client = connection.connect().unwrap();

    info!("Connected");
    info!("Registering Addition Procedure");
    client.register(URI::new("ca.test.add"), Box::new(addition_callback)).unwrap().await().unwrap();

    info!("Registering Multiplication Procedure");
    let mult_reg = client.register(URI::new("ca.test.mult"), Box::new(multiplication_callback)).unwrap().await().unwrap();

    info!("Unregistering Multiplication Procedure");
    client.unregister(mult_reg).unwrap().await().unwrap();

    info!("Registering Echo Procedure");
    client.register(URI::new("ca.test.echo"), Box::new(echo_callback)).unwrap().await().unwrap();

    println!("Press enter to quit");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    client.shutdown().unwrap().await().unwrap();
}
