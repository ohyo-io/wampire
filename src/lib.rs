//! Rust Wamp(v2) protocol library and router implementation
pub mod client;
mod error;
mod messages;
pub mod router;

use self::error::{Error, ErrorKind};

use crate::messages::{ErrorType, Message};
pub use crate::{
    client::{Client, Connection},
    messages::{
        ArgDict, ArgList, CallError, Dict, InvocationPolicy, List, MatchingPolicy, Reason, Value,
        URI,
    },
    router::Router,
};

pub type CallResult<T> = Result<T, CallError>;
pub type WampResult<T> = Result<T, Error>;
pub type ID = u64;
