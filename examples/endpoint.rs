#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use std::io;

use log::info;

use wampire::{client::Connection, ArgList, CallResult, Dict, List, Value, URI};

fn addition_callback(args: List, _kwargs: Dict) -> CallResult<(Option<List>, Option<Dict>)> {
    info!("Performing addition");
    args.verify_len(2)?;
    let a = args.get_int(0)?.unwrap();
    let b = args.get_int(1)?.unwrap();
    Ok((Some(vec![Value::Integer(a + b)]), None))
}

fn multiplication_callback(args: List, _kwargs: Dict) -> CallResult<(Option<List>, Option<Dict>)> {
    info!("Performing multiplication");
    args.verify_len(2)?;
    let a = args.get_int(0)?.unwrap();
    let b = args.get_int(1)?.unwrap();
    Ok((Some(vec![Value::Integer(a * b)]), None))
}

fn echo_callback(args: List, kwargs: Dict) -> CallResult<(Option<List>, Option<Dict>)> {
    info!("Performing echo");
    Ok((Some(args), Some(kwargs)))
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let connection = Connection::new("ws://127.0.0.1:8080/ws", "demo");
    info!("Connecting");
    let mut client = connection.connect().unwrap();

    info!("Connected");
    info!("Registering Addition Procedure");
    client
        .register(URI::new("ca.test.add"), Box::new(addition_callback))
        .await
        .unwrap();

    info!("Registering Multiplication Procedure");
    let mult_reg = client
        .register(URI::new("ca.test.mult"), Box::new(multiplication_callback))
        .await
        .unwrap();

    info!("Unregistering Multiplication Procedure");
    client.unregister(mult_reg).await.unwrap();

    info!("Registering Echo Procedure");
    client
        .register(URI::new("ca.test.echo"), Box::new(echo_callback))
        .await
        .unwrap();

    println!("Press enter to quit");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    client.shutdown().await.unwrap();
}
