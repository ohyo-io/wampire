use std::io;
use std::sync::{Arc, Mutex};

use env_logger;
use eventual::Async;
use log::info;

use wampire::client::{Client, Connection, Subscription};
use wampire::{MatchingPolicy, Value, URI};

enum Command {
    Sub,
    Pub,
    Unsub,
    List,
    Help,
    Quit,
    NoOp,
    Invalid(String),
}

fn print_prompt() {
    println!("Enter a command (or type \"help\")");
}

fn get_input_from_user() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

fn process_input(input: &str) -> (Command, Vec<String>) {
    let mut i_iter = input.splitn(2, ' ');
    let command = match i_iter.next() {
        Some(command) => command.trim().to_lowercase(),
        None => return (Command::NoOp, Vec::new()),
    };
    let command = match command.as_str() {
        "pub" => Command::Pub,
        "sub" => Command::Sub,
        "unsub" => Command::Unsub,
        "list" => Command::List,
        "help" => Command::Help,
        "quit" => Command::Quit,
        "" => Command::NoOp,
        x => Command::Invalid(x.to_string()),
    };
    let args = match i_iter.next() {
        Some(args_string) => args_string
            .split(',')
            .map(|s| s.trim().to_string())
            .collect(),
        None => Vec::new(),
    };
    (command, args)
}

fn subscribe(
    client: &mut Client,
    subscriptions: &mut Arc<Mutex<Vec<Subscription>>>,
    args: &[String],
) {
    if args.len() > 2 {
        println!("Too many arguments to subscribe.  Ignoring");
    } else if args.is_empty() {
        println!("Please specify the topic to subscribe to");
        return;
    }
    let topic = args[0].clone();
    let policy = if args.len() > 1 {
        match args[1].as_str() {
            "prefix" => MatchingPolicy::Prefix,
            "wild" => MatchingPolicy::Wildcard,
            "strict" => MatchingPolicy::Strict,
            _ => {
                println!("Invalid matching type, should be 'prefix', 'wild' or 'strict'");
                return;
            }
        }
    } else {
        MatchingPolicy::Strict
    };
    let subscriptions = Arc::clone(subscriptions);
    client
        .subscribe_with_pattern(
            URI::new(&topic),
            Box::new(move |args, kwargs| {
                println!(
                    "Received message on topic {} with args {:?} and kwargs {:?}",
                    topic, args, kwargs
                );
            }),
            policy,
        )
        .unwrap()
        .and_then(move |subscription| {
            println!("Subscribed to topic {}", subscription.topic.uri);
            subscriptions.lock().unwrap().push(subscription);
            Ok(())
        })
        .r#await()
        .unwrap();
}

fn unsubscribe(
    client: &mut Client,
    subscriptions: &mut Arc<Mutex<Vec<Subscription>>>,
    args: &[String],
) {
    if args.len() > 1 {
        println!("Too many arguments to subscribe.  Ignoring");
    } else if args.is_empty() {
        println!("Please specify the topic to subscribe to");
        return;
    }
    match args[0].parse::<usize>() {
        Ok(i) => {
            let mut subscriptions = subscriptions.lock().unwrap();
            if i >= subscriptions.len() {
                println!("Invalid subscription index: {}", i);
                return;
            }
            let subscription = subscriptions.remove(i);
            let topic = subscription.topic.uri.clone();
            client
                .unsubscribe(subscription)
                .unwrap()
                .and_then(move |()| {
                    println!("Successfully unsubscribed from {}", topic);
                    Ok(())
                })
                .r#await()
                .unwrap();
        }
        Err(_) => {
            println!("Invalid subscription index: {}", args[0]);
        }
    }
}

fn list(subscriptions: &Arc<Mutex<Vec<Subscription>>>) {
    let subscriptions = subscriptions.lock().unwrap();
    for (index, subscription) in subscriptions.iter().enumerate() {
        println!("{} {}", index, subscription.topic.uri);
    }
}

fn publish(client: &mut Client, args: &[String]) {
    if args.is_empty() {
        println!("Please specify a topic to publish to");
    }
    let uri = &args[0];
    let args = args[1..]
        .iter()
        .map(|arg| match arg.parse::<i64>() {
            Ok(i) => Value::Integer(i),
            Err(_) => Value::String(arg.clone()),
        })
        .collect();
    client
        .publish_and_acknowledge(URI::new(uri), Some(args), None)
        .unwrap()
        .r#await()
        .unwrap();
}

fn help() {
    println!("The following commands are supported:");
    println!("  sub <topic>, <matching policy>?",);
    println!("       Subscribes to the topic specified by the uri <topic>");
    println!("       <matching policy> specifies the type of patten matching used",);
    println!(
        "       <matching policy> should be one of 'strict' (the default), 'wild' or 'prefix'",
    );
    println!("  pub <topic>, <args>*",);
    println!("       Publishes to the topic specified by uri <topic>");
    println!("       <args> is an optinal, comma separated list of arguments");
    println!("  list");
    println!("       Lists all of the current subscriptions, along with their index");
    println!("  unsub <index>");
    println!("       Unsubscribes from the topic subscription specified by the given index");
    println!("  quit");
    println!("       Sends a goodbye message and quits the program");
}

fn event_loop(mut client: Client) {
    let mut subscriptions = Arc::new(Mutex::new(Vec::new()));
    loop {
        print_prompt();
        let input = get_input_from_user();
        let (command, args) = process_input(&input);
        match command {
            Command::Sub => subscribe(&mut client, &mut subscriptions, &args),
            Command::Pub => publish(&mut client, &args),
            Command::Unsub => unsubscribe(&mut client, &mut subscriptions, &args),
            Command::List => list(&subscriptions),
            Command::Help => help(),
            Command::Quit => break,
            Command::NoOp => {}
            Command::Invalid(bad_command) => print!("Invalid command: {}", bad_command),
        }
    }
    client.shutdown().unwrap().r#await().unwrap();
}

fn main() {
    env_logger::init();
    let connection = Connection::new("ws://127.0.0.1:8090/ws", "wampire_realm");
    info!("Connecting");
    let client = connection.connect().unwrap();

    info!("Connected");
    event_loop(client);
}
