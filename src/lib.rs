#![doc(html_logo_url = "https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/wampire.svg")]

#![warn(missing_docs)]

//! # Asynchronous implementation of Web Application Messaging Protocol (v2)
//!
//! WAMP is an open standard [WebSocket](https://tools.ietf.org/html/rfc6455) 
//! [subprotocol](https://www.iana.org/assignments/websocket/websocket.xml) 
//! that provides two application messaging patterns in one unified protocol:
//!
//! - routed **Remote Procedure Calls** and
//! - **Publish & Subscribe**
//!  
//! The WAMP protocol is a community effort and the specification is made available for 
//! free under an open license for everyone to use or implement.
//!
//! - [Introduction](crate::client)
//!   - [WebSocket](crate::client#websocket)
//!   - [WAMP](crate::client#wamp)
//! - [Message Routing in WAMP](crate::router)
//!   - [Loosely coupled](crate::router#loosely-coupled)
//!   - [Component based](crate::router#component-based)
//!   - [Real-time](crate::router#real-time)
//!   - [Language independent](crate::router#language-independent)
//! - [Protocol Specification](https://wamp-proto.org/spec.html)
//! - [WAMP compared](https://wamp-proto.org/comparison.html)
//! - [Implementations](https://wamp-proto.org/implementations.html)
//! - [Roadmap](https://wamp-proto.org/roadmap.html)
//! - [Frequently Asked Questions](https://wamp-proto.org/faq.html)
//!

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

/// Alias for call Result with [CallError]
pub type CallResult<T> = Result<T, CallError>;

/// Alias for Wamp Result
pub type WampResult<T> = Result<T, Error>;

/// Alias for u64
pub type ID = u64;
