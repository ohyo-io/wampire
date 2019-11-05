use std::io;

use log::info;
use env_logger;
use eventual::Async;

use wampire::client::{Client, Connection};
use wampire::{ArgList, Value, URI};

enum Command {
    Add,
    Echo,
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
        "add" => Command::Add,
        "echo" => Command::Echo,
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

#[allow(clippy::comparison_chain)]
fn add(client: &mut Client, args: &[String]) {
    if args.len() > 2 {
        println!("Too many arguments to add.  Ignoring");
    } else if args.len() < 2 {
        println!("Please pass two numbers for adding");
        return;
    }

    let a = match str::parse::<i64>(&args[0]) {
        Ok(i) => i,
        Err(_) => {
            println!("Please enter an integer (got {})", args[0]);
            return;
        }
    };
    let b = match str::parse::<i64>(&args[1]) {
        Ok(i) => i,
        Err(_) => {
            println!("Please enter an integer (got {})", args[0]);
            return;
        }
    };
    match client
        .call(
            URI::new("ca.test.add"),
            Some(vec![Value::Integer(a), Value::Integer(b)]),
            None,
        )
        .unwrap()
        .r#await()
    {
        Ok((args, _)) => {
            println!("Result: {}", args.get_int(0).unwrap().unwrap());
        }
        Err(e) => match e.take() {
            Some(e) => {
                println!("Error: {:?}", e);
            }
            None => {
                println!("Aborted");
            }
        },
    }
}

fn echo(client: &mut Client, args: Vec<String>) {
    let args = args.into_iter().map(Value::String).collect();
    let result = client
        .call(URI::new("ca.test.echo"), Some(args), None)
        .unwrap()
        .r#await();
    println!("Result: {:?}", result);
}

fn help() {
    println!("This client expects the 'endpoint' and 'router' examples to also be running",);
    println!("The following commands are supported:");
    println!("  add <a>, <b>");
    println!("     Adds the two numbers given by <a> and <b>",);
    println!("  echo <args>*");
    println!("     Echoes any arguments passed back");
    println!("  quit");
    println!("       Sends a goodbye message and quits the program");
}

fn event_loop(mut client: Client) {
    loop {
        print_prompt();
        let input = get_input_from_user();
        let (command, args) = process_input(&input);
        match command {
            Command::Add => add(&mut client, &args),
            Command::Echo => echo(&mut client, args),
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
    let connection = Connection::new("ws://127.0.0.1:8080/ws", "demo");
    info!("Connecting");
    let client = connection.connect().unwrap();

    info!("Connected");
    event_loop(client);
}
