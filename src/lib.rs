#![cfg_attr(feature = "cargo-clippy", allow(match_same_arms))]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate ws;
extern crate url;
extern crate rmp;
extern crate rmp_serde;
extern crate rand;
extern crate eventual;


#[macro_use]
extern crate log;

mod messages;
mod utils;
pub mod client;
pub mod router;

use ws::Error as WSError;
use std::fmt;
use url::ParseError;
use std::sync::mpsc::SendError;
use serde_json::Error as JSONError;
use rmp_serde::decode::Error as MsgPackError;

pub use messages::{URI, Dict, List, Value, Reason, MatchingPolicy, InvocationPolicy, CallError, ArgList, ArgDict};
use messages::{ErrorType, Message};
pub use client::{Client, Connection};
pub use router::Router;

pub type CallResult<T> = Result<T, CallError>;
pub type WampResult<T> = Result<T, Error>;
pub type ID = u64;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind
}

#[derive(Debug)]
pub enum ErrorKind {
    WSError(WSError),
    URLError(ParseError),
    HandshakeError(Reason),
    UnexpectedMessage(&'static str), // Used when a peer receives another message before Welcome or Hello
    ThreadError(SendError<messages::Message>),
    ConnectionLost,
    Closing(String),
    JSONError(JSONError),
    MsgPackError(MsgPackError),
    MalformedData,
    InvalidMessageType(Message),
    InvalidState(&'static str),
    Timeout,
    ErrorReason(ErrorType, ID, Reason),
}
impl Error {
    fn new(kind: ErrorKind) -> Error {
        Error {
            kind: kind
        }
    }

    fn get_description(&self) -> String {
        format!("WAMP Error: {}", self.kind.description())
    }

    #[inline]
    fn get_kind(self) -> ErrorKind{
        self.kind
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_description())
    }
}

impl ErrorKind {
    fn description(&self) -> String {
        match *self {
            ErrorKind::WSError(ref e) => e.to_string(),
            ErrorKind::URLError(ref e) => e.to_string(),
            ErrorKind::HandshakeError(ref r) => r.to_string(),
            ErrorKind::ThreadError(ref e) => e.to_string(),
            ErrorKind::JSONError(ref e) => e.to_string(),
            ErrorKind::MsgPackError(ref e) => e.to_string(),
            ErrorKind::ErrorReason(_, _, ref s) => s.to_string(),
            ErrorKind::Closing(ref s) => s.clone(),
            ErrorKind::UnexpectedMessage(s) |
            ErrorKind::InvalidState(s) => s.to_string(),
            ErrorKind::ConnectionLost => "Connection Lost".to_string(),
            ErrorKind::MalformedData => "Malformed Data".to_string(),
            ErrorKind::Timeout => "Connection timed out".to_string(),
            ErrorKind::InvalidMessageType(ref t) => format!("Invalid Message Type: {:?}", t),
        }
    }
}
