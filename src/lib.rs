pub mod client;
mod error;
mod messages;
pub mod router;
mod utils;

use self::error::{Error, ErrorKind};

pub use crate::client::{Client, Connection};
pub use crate::messages::{
    ArgDict, ArgList, CallError, Dict, InvocationPolicy, List, MatchingPolicy, Reason, Value, URI,
};
use crate::messages::{ErrorType, Message};
pub use crate::router::Router;

pub type CallResult<T> = Result<T, CallError>;
pub type WampResult<T> = Result<T, Error>;
pub type ID = u64;
