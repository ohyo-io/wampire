#![allow(dead_code, unused_imports, unused_variables)]
use futures::channel::oneshot::*;

#[derive(Debug)]
enum SourceError {
    Simple,
}

#[derive(Debug)]
enum TargetError {
    Complex,
}

#[tokio::main]
async fn main() {
    let (tx, rx) = channel();
    let handle = tokio::spawn(async move {
        match rx.await {
            Ok(val) => println!("Got result {}", val),
            Err(err) => println!("Gotcha {:?}", err)
        }
    });

    // let source: Result<bool, SourceError> = Err(SourceError::Simple);
    let source: Result<bool, SourceError> = Ok(true);
    let target = source.map_err(|_| TargetError::Complex);

    let source: Result<Result<bool, SourceError>, TargetError> = Ok(Ok(true));
    let target = source.map(|a| {1});
    // let target = source.map_err(|_| TargetError::Complex);

    let source: Result<Result<bool, SourceError>, TargetError> = Ok(Ok(true));
    let target = source.unwrap_or(Err(SourceError::Simple));

    println!("Here {:?}", target);

    let _ = tx.send(123);
    let _ = handle.await;

    // let a = async move {
    //     Ok(123)
    // };
}
