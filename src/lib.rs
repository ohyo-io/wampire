#![cfg_attr(feature = "cargo-clippy", allow(match_same_arms))]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate eventual;
extern crate itertools;
extern crate rand;
extern crate rmp;
extern crate rmp_serde;
extern crate url;
extern crate ws;

#[macro_use]
extern crate log;

pub mod client;
mod error;
mod messages;
pub mod router;
mod utils;

use self::error::*;

pub use client::{Client, Connection};
pub use messages::{ArgDict, ArgList, CallError, Dict, InvocationPolicy, List, MatchingPolicy,
                   Reason, Value, URI};
use messages::{ErrorType, Message};
pub use router::Router;

pub type CallResult<T> = Result<T, CallError>;
pub type WampResult<T> = Result<T, Error>;
pub type ID = u64;
