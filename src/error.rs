use std::fmt;
use std::sync::mpsc::SendError;

use rmp_serde::decode::Error as MsgPackError;
use serde_json::Error as JSONError;
use url::ParseError;
use ws::Error as WSError;

use crate::messages::{self, Reason};

use super::{ErrorType, Message, ID};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

#[allow(clippy::large_enum_variant)]
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
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }

    fn get_description(&self) -> String {
        format!("WAMP Error: {}", self.kind.description())
    }

    #[inline]
    pub fn get_kind(self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_description())
    }
}

impl ErrorKind {
    pub fn description(&self) -> String {
        match *self {
            ErrorKind::WSError(ref e) => e.to_string(),
            ErrorKind::URLError(ref e) => e.to_string(),
            ErrorKind::HandshakeError(ref r) => r.to_string(),
            ErrorKind::ThreadError(ref e) => e.to_string(),
            ErrorKind::JSONError(ref e) => e.to_string(),
            ErrorKind::MsgPackError(ref e) => e.to_string(),
            ErrorKind::ErrorReason(_, _, ref s) => s.to_string(),
            ErrorKind::Closing(ref s) => s.clone(),
            ErrorKind::UnexpectedMessage(s) | ErrorKind::InvalidState(s) => s.to_string(),
            ErrorKind::ConnectionLost => "Connection Lost".to_string(),
            ErrorKind::MalformedData => "Malformed Data".to_string(),
            ErrorKind::Timeout => "Connection timed out".to_string(),
            ErrorKind::InvalidMessageType(ref t) => format!("Invalid Message Type: {:?}", t),
        }
    }
}
