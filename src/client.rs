//! ## WebSocket[](#websocket "Permalink to this headline")
//! 
//! The WebSocket protocol brings bi-directional (soft) real-time and wire traffic efficient connections to the browser. 
//! Today (2018) WebSocket is universally supported in browsers, network equipment, servers and client languages.
//! 
//! Despite having opened completely new possibilities on the Web, WebSocket defines an API for 
//! application developers at the *message* level, and *point-to-point*, requiring users who want to use 
//! WebSocket connections in their applications to define their own semantics on top of it.
//! 
//! The Web Application Messaging Protocol (WAMP) aims to provide application developers with the right level of semantics, 
//! with what they need to handle messaging and communication between components in distributed applications at 
//! a convenient and abstracted way.
//! 
//! WAMP was initially defined as a WebSocket sub-protocol, which provided **Publish & Subscribe (PubSub)** functionality 
//! as well as **routed Remote Procedure Calls (rRPC)** for procedures implemented in a WAMP router. 
//! Feedback from implementers and users of this was included in a second version of the protocol which this document defines. 
//! Among the changes was that WAMP can now run over any transport which is message-oriented, ordered, reliable, and bi-directional.
//! 
//! > If you want to read more about WebSocket, we recommend two blog posts of the creators of WAMP;)
//! > 
//! > - [WebSocket - Why, what, and - can I use it?][1]
//! > - [Dissecting WebSocket’s Overhead][2]
//!   
//! ## WAMP[](#wamp "Permalink to this headline")
//! 
//! WAMP is a routed protocol, with all components connecting to a WAMP Router, where the WAMP Router performs message 
//! routing between the components, and provides two messaging patterns in one Web native protocol:
//! 
//! - **Publish & Subscribe (PubSub)** and
//! - routed **Remote Procedure Calls (rRPC)**
//!   
//! Publish & Subscribe (PubSub) is an established messaging pattern where a component, the Subscriber, informs the router 
//! that it wants to receive information on a topic (i.e., it subscribes to a topic). Another component, a Publisher, 
//! can then publish to this topic, and the router distributes events to all Subscribers.
//! 
//! Routed Remote Procedure Calls (rRPCs) rely on the same sort of decoupling that is used by the Publish & Subscribe pattern. 
//! A component, the Callee, announces to the router that it provides a certain procedure, identified by a procedure name. 
//! Other components, Callers, can then call the procedure, with the router invoking the procedure on the Callee, 
//! receiving the procedure’s result, and then forwarding this result back to the Caller. Routed RPCs differ from 
//! traditional client-server RPCs in that the router serves as an intermediary between the Caller and the Callee.
//! 
//! **Advantages of decoupling and routed RPCs**
//! 
//! The decoupling in routed RPCs arises from the fact that the Caller is no longer required to have knowledge of the Callee; 
//! it merely needs to know the identifier of the procedure it wants to call. There no longer is a need for a direct network connection 
//! or path between the caller and the callee, since all messages are routed at the WAMP level.
//! 
//! This approach enables a whole range of possibilities:
//! 
//! - calling into procedures in components which are not reachable from outside at the network level (e.g. on a NATted connection), 
//! but which can establish an outgoing network connection to the WAMP router.
//!   
//! - This decoupling of transport and application layer traffic allows a “reversal of command” where a 
//! cloud-based system can securely control remote devices
//! - It also allows to treat frontend and backend components (microservices) the same, and it even allows 
//! to develop backend code in the browser ([Free Your Code - Backends in the Browser][3]).
//! - Since no ports on edge devices need to be opened for WAMP to work (in both directions), 
//! the remote attack surface of these (potentially many) devices is completely closed ([Security in the IoT][4]).
//!   
//! - Finally, since the Caller is not aware where, or even who is processing the call (and it should not care!), 
//! it is easily possible to make application components highly-available (using hot standby components) 
//! or scale-out application components ([Scaling microservices with Crossbar.io][5]).
//!   
//! **Summary**
//! 
//! Combining the Publish & Subscribe and routed Remote Procedure Calls in one Web native, real-time transport protocol (WebSocket) 
//! allows WAMP to be used for the entire messaging requirements of component and microservice based applications, reducing technology 
//! stack complexity and overhead, providing a capable and secure fundament for applications to rely on.
//! 
//! [1]: https://crossbario.com/blog/Websocket-Why-What-Can-I-Use-It/
//! [2]: https://crossbario.com/blog/Dissecting-Websocket-Overhead/
//! [3]: https://crossbario.com/blog/Free-Your-Code-Backends-in-the-Browser/
//! [4]: https://crossbario.com/static/presentations/iot-security/index.html
//! [5]: https://crossbario.com/static/presentations/microservices/index.html

#![allow(dead_code)]
#![allow(unused_imports)]
use std::{
    collections::HashMap,
    fmt,
    io::Cursor,
    pin::Pin,
    sync::{
        mpsc::{channel, Sender as CHSender},
        Arc, Mutex, MutexGuard,
    },
    thread,
};

use futures::{channel::oneshot, Future};
use intmap::IntMap;
use log::{debug, error, info, trace, warn};
use rmp_serde::{Deserializer as RMPDeserializer, Serializer};
use serde::{Deserialize, Serialize};
use url::Url;
use ws::{
    connect, util::Token, CloseCode, Error as WSError, ErrorKind as WSErrorKind, Handler,
    Handshake, Message as WSMessage, Request, Result as WSResult, Sender,
};

use crate::{
    messages::{
        CallOptions, ClientRoles, Dict, ErrorDetails, ErrorType, HelloDetails, InvocationDetails,
        List, MatchingPolicy, Message, PublishOptions, Reason, RegisterOptions, ResultDetails,
        SubscribeOptions, WelcomeDetails, YieldOptions, URI,
    },
    CallError, CallResult, Error, ErrorKind, WampResult, ID,
};

const CONNECTION_TIMEOUT: Token = Token(124);

/// Represents WAMP connection
pub struct Connection {
    realm: URI,
    url: String,
}

/// Represents WAMP subcription
pub struct Subscription {
    /// Topic URI
    pub topic: URI,
    subscription_id: ID,
}

/// Represents WAMP registration
pub struct Registration {
    /// Procedure URI
    pub procedure: URI,
    registration_id: ID,
}

struct SubscriptionCallbackWrapper {
    callback: Box<dyn FnMut(List, Dict)>,
}

struct RegistrationCallbackWrapper {
    callback: Callback,
}

type Complete<T> = oneshot::Sender<Result<T, CallError>>;

/// Alias for WAMP callback
pub type Callback = Box<dyn FnMut(List, Dict) -> CallResult<(Option<List>, Option<Dict>)>>;

static WAMP_JSON: &str = "wamp.2.json";
static WAMP_MSGPACK: &str = "wamp.2.msgpack";

#[derive(PartialEq, Debug)]
enum ConnectionState {
    Connecting,
    Connected,
    ShuttingDown,
    Disconnected,
}

type ConnectionResult = Result<Arc<Mutex<ConnectionInfo>>, Error>;

unsafe impl<'a> Send for ConnectionInfo {}

unsafe impl<'a> Sync for ConnectionInfo {}

unsafe impl<'a> Send for SubscriptionCallbackWrapper {}

unsafe impl<'a> Sync for SubscriptionCallbackWrapper {}

unsafe impl<'a> Send for RegistrationCallbackWrapper {}

unsafe impl<'a> Sync for RegistrationCallbackWrapper {}

/// Represents WAMP Client
pub struct Client {
    connection_info: Arc<Mutex<ConnectionInfo>>,
    max_session_id: ID,
}

/// Represents connection handler
pub struct ConnectionHandler {
    connection_info: Arc<Mutex<ConnectionInfo>>,
    realm: URI,
    state_transmission: CHSender<ConnectionResult>,
}

struct ConnectionInfo {
    connection_state: ConnectionState,
    sender: Sender,
    subscription_requests: IntMap<(Complete<Subscription>, SubscriptionCallbackWrapper, URI)>,
    unsubscription_requests: IntMap<(Complete<()>, ID)>,
    subscriptions: IntMap<SubscriptionCallbackWrapper>,
    registrations: IntMap<RegistrationCallbackWrapper>,
    call_requests: IntMap<Complete<(List, Dict)>>,
    registration_requests: IntMap<(Complete<Registration>, RegistrationCallbackWrapper, URI)>,
    unregistration_requests: IntMap<(Complete<()>, ID)>,
    protocol: String,
    publish_requests: IntMap<Complete<ID>>,
    shutdown_complete: Option<Complete<()>>,
    session_id: ID,
}

trait MessageSender {
    fn send_message(&self, message: Message) -> WampResult<()>;
}

impl MessageSender for ConnectionInfo {
    fn send_message(&self, message: Message) -> WampResult<()> {
        debug!("Sending message {:?} via {}", message, self.protocol);
        let send_result = if self.protocol == WAMP_JSON {
            // Send the json message
            self.sender
                .send(WSMessage::Text(serde_json::to_string(&message).unwrap()))
        } else {
            // Send the msgpack
            let mut buf: Vec<u8> = Vec::new();

            message
                .serialize(&mut Serializer::new(&mut buf).with_struct_map())
                .unwrap();

            self.sender.send(WSMessage::Binary(buf))
        };
        match send_result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::new(ErrorKind::WSError(e))),
        }
    }
}

impl Connection {
    /// Create new connection with uri and realm
    pub fn new(url: &str, realm: &str) -> Connection {
        Connection {
            realm: URI::new(realm),
            url: url.to_string(),
        }
    }

    /// Connect to router
    pub fn connect(&self) -> WampResult<Client> {
        let (tx, rx) = channel();
        let url = self.url.clone();
        let realm = self.realm.clone();
        thread::spawn(move || {
            trace!("Beginning Connection");
            let connect_result = connect(url, |out| {
                trace!("Got sender");
                // Set up timeout
                out.timeout(5000, CONNECTION_TIMEOUT).unwrap();
                let info = Arc::new(Mutex::new(ConnectionInfo {
                    protocol: String::new(),
                    subscription_requests: IntMap::new(),
                    unsubscription_requests: IntMap::new(),
                    subscriptions: IntMap::new(),
                    registrations: IntMap::new(),
                    call_requests: IntMap::new(),
                    registration_requests: IntMap::new(),
                    unregistration_requests: IntMap::new(),
                    sender: out,
                    connection_state: ConnectionState::Connecting,
                    publish_requests: IntMap::new(),
                    shutdown_complete: None,
                    session_id: 0,
                }));

                ConnectionHandler {
                    state_transmission: tx.clone(),
                    connection_info: info,
                    realm: realm.clone(),
                }
            })
            .map_err(|e| Error::new(ErrorKind::WSError(e)));
            debug!("Result of connection: {:?}", connect_result);
            match connect_result {
                Ok(_) => (),
                Err(e) => {
                    tx.send(Err(e)).unwrap();
                }
            }
        });
        let info = rx.recv().unwrap()?;
        Ok(Client {
            connection_info: info,
            max_session_id: 0,
        })
    }
}

macro_rules! cancel_future_tuple {
    ($dict:expr) => {{
        for (_, future) in $dict.drain() {
            let _ = future
                .0
                .send(Err(CallError::new(Reason::NetworkFailure, None, None)));
        }
    }};
}

macro_rules! cancel_future {
    ($dict:expr) => {{
        for (_, future) in $dict.drain() {
            let _ = future.send(Err(CallError::new(Reason::NetworkFailure, None, None)));
        }
    }};
}

impl Handler for ConnectionHandler {
    fn on_open(&mut self, handshake: Handshake) -> WSResult<()> {
        debug!("Connection Opened");
        let mut info = self.connection_info.lock().unwrap();
        info.protocol = match handshake.response.protocol()? {
            Some(protocol) => protocol.to_string(),
            None => {
                warn!("Router did not specify protocol. Defaulting to wamp.2.json");
                WAMP_JSON.to_string()
            }
        };

        let hello_message =
            Message::Hello(self.realm.clone(), HelloDetails::new(ClientRoles::new()));

        debug!("Sending Hello message");
        match info.send_message(hello_message) {
            Ok(_) => Ok(()),
            Err(e) => {
                if let ErrorKind::WSError(e) = e.kind {
                    Err(e)
                } else {
                    Err(WSError::new(WSErrorKind::Internal, "Unknown error"))
                }
            }
        }
    }

    fn on_message(&mut self, message: WSMessage) -> WSResult<()> {
        debug!("Server sent a message: {:?}", message);
        match message {
            WSMessage::Text(message) => match serde_json::from_str(&message) {
                Ok(message) => {
                    if !self.handle_message(message) {
                        return self.connection_info.lock().unwrap().sender.shutdown();
                    }
                }
                Err(_) => {
                    error!("Received unknown message: {}", message);
                    return Ok(());
                }
            },
            WSMessage::Binary(message) => {
                let mut de = RMPDeserializer::new(Cursor::new(&*message));
                match Deserialize::deserialize(&mut de) {
                    Ok(message) => {
                        if !self.handle_message(message) {
                            return self.connection_info.lock().unwrap().sender.shutdown();
                        }
                    }
                    Err(_) => {
                        error!("Could not understand MsgPack message");
                    }
                }
            }
        }
        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str) {
        debug!("Closing connection");
        let mut info = self.connection_info.lock().unwrap();
        info.sender.close(CloseCode::Normal).ok();
        info.connection_state = ConnectionState::Disconnected;
        cancel_future_tuple!(info.subscription_requests);
        cancel_future_tuple!(info.unsubscription_requests);
        cancel_future_tuple!(info.registration_requests);
        cancel_future_tuple!(info.unregistration_requests);
        cancel_future!(info.publish_requests);
        cancel_future!(info.call_requests);
        info.sender.shutdown().ok();

        if let Some(promise) = info.shutdown_complete.take() {
            let _ = promise.send(Ok(()));
        }
    }

    fn on_timeout(&mut self, token: Token) -> WSResult<()> {
        if token == CONNECTION_TIMEOUT {
            let info = self.connection_info.lock().unwrap();
            if info.connection_state == ConnectionState::Connecting {
                info.sender.shutdown().unwrap();
                drop(info);
                self.state_transmission
                    .send(Err(Error::new(ErrorKind::Timeout)))
                    .unwrap();
            }
        }
        Ok(())
    }

    fn build_request(&mut self, url: &Url) -> WSResult<Request> {
        trace!("Building request");
        let mut request = Request::from_url(url)?;
        request.add_protocol(WAMP_MSGPACK);
        request.add_protocol(WAMP_JSON);
        Ok(request)
    }
}

impl ConnectionHandler {
    fn handle_message(&mut self, message: Message) -> bool {
        let mut info = self.connection_info.lock().unwrap();
        debug!(
            "Processing message from server (state: {:?})",
            info.connection_state
        );
        match info.connection_state {
            ConnectionState::Connecting => match message {
                Message::Welcome(session_id, details) => {
                    self.handle_welcome(info, session_id, details)
                }
                Message::Abort(_, reason) => {
                    self.handle_abort(info, reason);
                    return false;
                }
                _ => return false,
            },
            ConnectionState::Connected => {
                debug!("Received a message from the server: {:?}", message);
                match message {
                    Message::Subscribed(request_id, subscription_id) => {
                        self.handle_subscribed(info, request_id, subscription_id)
                    }
                    Message::Unsubscribed(request_id) => self.handle_unsubscribed(info, request_id),
                    Message::Event(subscription_id, _, _, args, kwargs) => {
                        self.handle_event(info, subscription_id, args, kwargs)
                    }
                    Message::Published(request_id, publication_id) => {
                        self.handle_published(info, request_id, publication_id)
                    }
                    Message::Registered(request_id, registration_id) => {
                        self.handle_registered(info, request_id, registration_id)
                    }
                    Message::Unregistered(request_id) => self.handle_unregistered(info, request_id),
                    Message::Invocation(request_id, registration_id, details, args, kwargs) => self
                        .handle_invocation(
                            info,
                            request_id,
                            registration_id,
                            details,
                            args,
                            kwargs,
                        ),
                    Message::Result(call_id, details, args, kwargs) => {
                        self.handle_result(info, call_id, details, args, kwargs)
                    }
                    Message::Error(e_type, request_id, details, reason, args, kwargs) => {
                        self.handle_error(info, e_type, request_id, details, reason, args, kwargs)
                    }
                    Message::Goodbye(_, reason) => {
                        self.handle_goodbye(info, reason);
                        return false;
                    }
                    _ => warn!("Received unknown message.  Ignoring. {:?}", message),
                }
            }
            ConnectionState::ShuttingDown => {
                if let Message::Goodbye(_, _) = message {
                    // The router has seen our goodbye message and has responded in kind
                    info!("Router acknowledged disconnect");
                    if let Some(promise) = info.shutdown_complete.take() {
                        let _ = promise.send(Ok(()));
                    }
                    return false;
                } else {
                    warn!(
                        "Received message after shutting down, ignoring: {:?}",
                        message
                    );
                    return false;
                }
            }
            ConnectionState::Disconnected => {
                // Should never happen
                return false;
            }
        }
        true
    }

    fn handle_subscribed(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        request_id: ID,
        subscription_id: ID,
    ) {
        // TODO handle errors here
        info!("Received a subscribed notification");
        match info.subscription_requests.remove(request_id) {
            Some((promise, callback, topic)) => {
                debug!("Completing promise");
                let subscription = Subscription {
                    topic,
                    subscription_id,
                };
                info.subscriptions.insert(subscription_id, callback);
                drop(info);
                let _ = promise.send(Ok(subscription));
            }
            None => {
                warn!(
                    "Received a subscribed notification for a subscription we don't have.  ID: {}",
                    request_id
                );
            }
        }
    }

    fn handle_subscribe_error(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        warn!("Received an error for a subscription");
        match info.subscription_requests.remove(request_id) {
            Some((promise, _, _)) => {
                drop(info);
                let _ = promise.send(Err(CallError::new(reason, args, kwargs)));
            }
            None => {
                warn!(
                    "Received a an error notification for a request we didn't make.  ID: {}",
                    request_id
                );
            }
        }
    }

    fn handle_unsubscribed(&self, mut info: MutexGuard<'_, ConnectionInfo>, request_id: ID) {
        match info.unsubscription_requests.remove(request_id) {
            Some((promise, subscription_id)) => {
                info.unsubscription_requests.remove(subscription_id);
                drop(info);
                let _ = promise.send(Ok(()));
            }
            None => {
                warn!("Received a unsubscribed notification for a subscription we don't have.  ID: {}", request_id);
            }
        }
    }

    fn handle_unsubscribe_error(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        match info.unsubscription_requests.remove(request_id) {
            Some((promise, subscription_id)) => {
                info.unsubscription_requests.remove(subscription_id);
                drop(info);
                let _ = promise.send(Err(CallError::new(reason, args, kwargs)));
            }
            None => {
                warn!(
                    "Received a unsubscribed error for a subscription we don't have.  ID: {}",
                    request_id
                );
            }
        }
    }

    fn handle_registered(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        request_id: ID,
        registration_id: ID,
    ) {
        // TODO handle errors here
        info!("Received a registered notification");
        match info.registration_requests.remove(request_id) {
            Some((promise, callback, procedure)) => {
                info.registrations.insert(registration_id, callback);
                drop(info);
                let registration = Registration {
                    procedure,
                    registration_id,
                };
                let _ = promise.send(Ok(registration));
            }
            None => {
                warn!(
                    "Received a registered notification for a registration we don't have.  ID: {}",
                    request_id
                );
            }
        }
    }

    fn handle_register_error(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        info!("Received a registration error");
        match info.registration_requests.remove(request_id) {
            Some((promise, _, _)) => {
                drop(info);
                let _ = promise.send(Err(CallError::new(reason, args, kwargs)));
            }
            None => {
                warn!(
                    "Received a registered error for a registration we don't have.  ID: {}",
                    request_id
                );
            }
        }
    }

    fn handle_unregistered(&self, mut info: MutexGuard<'_, ConnectionInfo>, request_id: ID) {
        match info.unregistration_requests.remove(request_id) {
            Some((promise, registration_id)) => {
                info.registrations.remove(registration_id);
                drop(info);
                let _ = promise.send(Ok(()));
            }
            None => {
                warn!("Received a unregistered notification for a registration we don't have.  ID: {}", request_id);
            }
        }
    }

    fn handle_unregister_error(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        match info.unregistration_requests.remove(request_id) {
            Some((promise, _)) => {
                drop(info);
                let _ = promise.send(Err(CallError::new(reason, args, kwargs)));
            }
            None => {
                warn!(
                    "Received a unregistered error for a registration we don't have.  ID: {}",
                    request_id
                );
            }
        }
    }

    fn handle_published(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        request_id: ID,
        publication_id: ID,
    ) {
        match info.publish_requests.remove(request_id) {
            Some(promise) => {
                let _ = promise.send(Ok(publication_id));
            }
            None => warn!(
                "Received published notification for a request we weren't tracking: {}",
                request_id
            ),
        }
    }
    fn handle_publish_error(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        match info.publish_requests.remove(request_id) {
            Some(promise) => {
                let _ = promise.send(Err(CallError::new(reason, args, kwargs)));
            }
            None => warn!("Received published error for a publication: {}", request_id),
        }
    }

    fn handle_welcome(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        session_id: ID,
        _details: WelcomeDetails,
    ) {
        info.session_id = session_id;
        info.connection_state = ConnectionState::Connected;
        drop(info);
        self.state_transmission
            .send(Ok(Arc::clone(&self.connection_info)))
            .unwrap();
    }

    fn handle_abort(&self, mut info: MutexGuard<'_, ConnectionInfo>, reason: Reason) {
        error!("Router aborted connection.  Reason: {:?}", reason);
        info.connection_state = ConnectionState::ShuttingDown;
    }

    fn handle_event(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        subscription_id: ID,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        let args = args.unwrap_or_default();
        let kwargs = kwargs.unwrap_or_default();
        match info.subscriptions.get_mut(subscription_id) {
            Some(subscription) => {
                let callback = &mut subscription.callback;
                callback(args, kwargs);
            }
            None => {
                warn!(
                    "Received an event for a subscription we don't have.  ID: {}",
                    subscription_id
                );
            }
        }
    }

    fn handle_invocation(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        request_id: ID,
        registration_id: ID,
        _details: InvocationDetails,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        let args = args.unwrap_or_default();
        let kwargs = kwargs.unwrap_or_default();
        let message = match info.registrations.get_mut(registration_id) {
            Some(registration) => {
                let callback = &mut registration.callback;
                match callback(args, kwargs) {
                    Ok((rargs, rkwargs)) => {
                        Message::Yield(request_id, YieldOptions::new(), rargs, rkwargs)
                    }
                    Err(error) => {
                        let (reason, args, kwargs) = error.into_tuple();
                        Message::Error(
                            ErrorType::Invocation,
                            request_id,
                            HashMap::new(),
                            reason,
                            args,
                            kwargs,
                        )
                    }
                }
            }
            None => {
                warn!(
                    "Received an invocation for a procedure we don't have.  ID: {}",
                    registration_id
                );
                return;
            }
        };
        info.send_message(message).ok();
    }

    fn handle_result(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        call_id: ID,
        _details: ResultDetails,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        let args = args.unwrap_or_default();
        let kwargs = kwargs.unwrap_or_default();
        match info.call_requests.remove(call_id) {
            Some(promise) => {
                let _ = promise.send(Ok((args, kwargs)));
            }
            None => {
                warn!(
                    "Received a result for a call we didn't make.  ID: {}",
                    call_id
                );
            }
        }
    }

    fn handle_call_error(
        &self,
        mut info: MutexGuard<'_, ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        match info.call_requests.remove(request_id) {
            Some(promise) => {
                let _ = promise.send(Err(CallError::new(reason, args, kwargs)));
            }
            None => {
                warn!(
                    "Received an error for a call we didn't make.  ID: {}",
                    request_id
                );
            }
        }
    }

    fn handle_goodbye(&self, mut info: MutexGuard<'_, ConnectionInfo>, reason: Reason) {
        info!("Router said goodbye.  Reason: {:?}", reason);

        info.send_message(Message::Goodbye(ErrorDetails::new(), Reason::GoodbyeAndOut))
            .unwrap();
        info.connection_state = ConnectionState::ShuttingDown;
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_error(
        &self,
        info: MutexGuard<'_, ConnectionInfo>,
        e_type: ErrorType,
        request_id: ID,
        _details: Dict,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        match e_type {
            ErrorType::Subscribe => {
                self.handle_subscribe_error(info, request_id, reason, args, kwargs)
            }
            ErrorType::Unsubscribe => {
                self.handle_unsubscribe_error(info, request_id, reason, args, kwargs)
            }
            ErrorType::Publish => self.handle_publish_error(info, request_id, reason, args, kwargs),
            ErrorType::Register => {
                self.handle_register_error(info, request_id, reason, args, kwargs)
            }
            ErrorType::Unregister => {
                self.handle_unregister_error(info, request_id, reason, args, kwargs)
            }
            ErrorType::Invocation => {
                warn!("Received an error for an invocation message, which we did not (and could not) send")
            }
            ErrorType::Call => self.handle_call_error(info, request_id, reason, args, kwargs),
        }
    }
}

impl Client {
    fn get_next_session_id(&mut self) -> ID {
        self.max_session_id += 1;
        self.max_session_id
    }

    /// Send a subscribe messages
    pub fn subscribe_with_pattern(
        &mut self,
        topic_pattern: URI,
        callback: Box<dyn FnMut(List, Dict)>,
        policy: MatchingPolicy,
    ) -> Pin<Box<dyn Future<Output = Result<Subscription, CallError>>>> {
        let request_id = self.get_next_session_id();

        let (complete, receiver) = oneshot::channel();

        let callback = SubscriptionCallbackWrapper { callback };
        let mut options = SubscribeOptions::new();

        if policy != MatchingPolicy::Strict {
            options.pattern_match = policy
        }

        let mut info = self.connection_info.lock().unwrap();
        info.subscription_requests
            .insert(request_id, (complete, callback, topic_pattern.clone()));

        info.send_message(Message::Subscribe(request_id, options, topic_pattern))
            .unwrap();

        Box::pin(async {
            receiver.await.unwrap_or(Err(CallError {
                reason: Reason::InternalError,
                args: None,
                kwargs: None,
            }))
        })
    }

    /// Subscribe to topic
    pub fn subscribe(
        &mut self,
        topic: URI,
        callback: Box<dyn FnMut(List, Dict)>,
    ) -> Pin<Box<dyn Future<Output = Result<Subscription, CallError>>>> {
        self.subscribe_with_pattern(topic, callback, MatchingPolicy::Strict)
    }

    /// Send a register message
    pub fn register_with_pattern(
        &mut self,
        procedure_pattern: URI,
        callback: Callback,
        policy: MatchingPolicy,
    ) -> Pin<Box<dyn Future<Output = Result<Registration, CallError>>>> {
        let request_id = self.get_next_session_id();

        let (complete, receiver) = oneshot::channel();

        let callback = RegistrationCallbackWrapper { callback };
        let mut options = RegisterOptions::new();

        if policy != MatchingPolicy::Strict {
            options.pattern_match = policy
        }

        debug!("Acquiring lock on connection info");
        let mut info = self.connection_info.lock().unwrap();

        debug!("Lock on connection info acquired");
        info.registration_requests
            .insert(request_id, (complete, callback, procedure_pattern.clone()));

        info.send_message(Message::Register(request_id, options, procedure_pattern))
            .unwrap();

        Box::pin(async {
            receiver.await.unwrap_or(Err(CallError {
                reason: Reason::InternalError,
                args: None,
                kwargs: None,
            }))
        })
    }

    /// Register procedure with callback
    pub fn register(
        &mut self,
        procedure: URI,
        callback: Callback,
    ) -> Pin<Box<dyn Future<Output = Result<Registration, CallError>>>> {
        self.register_with_pattern(procedure, callback, MatchingPolicy::Strict)
    }

    /// Unsubscribe from topic
    pub fn unsubscribe(
        &mut self,
        subscription: Subscription,
    ) -> Pin<Box<dyn Future<Output = Result<(), CallError>>>> {
        let request_id = self.get_next_session_id();

        let mut info = self.connection_info.lock().unwrap();

        info.send_message(Message::Unsubscribe(
            request_id,
            subscription.subscription_id,
        ))
        .unwrap();

        let (complete, receiver) = oneshot::channel();

        info.unsubscription_requests
            .insert(request_id, (complete, subscription.subscription_id));

        Box::pin(async {
            receiver.await.unwrap_or(Err(CallError {
                reason: Reason::InternalError,
                args: None,
                kwargs: None,
            }))
        })
    }

    /// Unregister procedure 
    pub fn unregister(
        &mut self,
        registration: Registration,
    ) -> Pin<Box<dyn Future<Output = Result<(), CallError>>>> {
        let request_id = self.get_next_session_id();

        let mut info = self.connection_info.lock().unwrap();

        info.send_message(Message::Unregister(
            request_id,
            registration.registration_id,
        ))
        .unwrap();

        let (complete, receiver) = oneshot::channel();

        info.unregistration_requests
            .insert(request_id, (complete, registration.registration_id));

        Box::pin(async {
            receiver.await.unwrap_or(Err(CallError {
                reason: Reason::InternalError,
                args: None,
                kwargs: None,
            }))
        })
    }

    /// Publish to topic
    pub fn publish(
        &mut self,
        topic: URI,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) -> WampResult<()> {
        info!("Publishing to {:?} with {:?} | {:?}", topic, args, kwargs);

        let request_id = self.get_next_session_id();

        let info = self.connection_info.lock().unwrap();

        info.send_message(Message::Publish(
            request_id,
            PublishOptions::new(false),
            topic,
            args,
            kwargs,
        ))
    }

    /// Call the procedure
    pub fn call(
        &mut self,
        procedure: URI,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) -> Pin<Box<dyn Future<Output = Result<(List, Dict), CallError>>>> {
        info!("Calling {:?} with {:?} | {:?}", procedure, args, kwargs);

        let request_id = self.get_next_session_id();

        let (complete, receiver) = oneshot::channel();

        let mut info = self.connection_info.lock().unwrap();

        info.call_requests.insert(request_id, complete);

        info.send_message(Message::Call(
            request_id,
            CallOptions::new(),
            procedure,
            args,
            kwargs,
        ))
        .unwrap();

        Box::pin(async {
            receiver.await.unwrap_or(Err(CallError {
                reason: Reason::InternalError,
                args: None,
                kwargs: None,
            }))
        })
    }

    /// Publish to topic and acknowledge
    pub fn publish_and_acknowledge(
        &mut self,
        topic: URI,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) -> Pin<Box<dyn Future<Output = Result<ID, CallError>>>> {
        info!("Publishing to {:?} with {:?} | {:?}", topic, args, kwargs);

        let request_id = self.get_next_session_id();

        let (complete, receiver) = oneshot::channel();

        let mut info = self.connection_info.lock().unwrap();

        info.publish_requests.insert(request_id, complete);

        info.send_message(Message::Publish(
            request_id,
            PublishOptions::new(true),
            topic,
            args,
            kwargs,
        ))
        .unwrap();

        Box::pin(async {
            receiver.await.unwrap_or(Err(CallError {
                reason: Reason::InternalError,
                args: None,
                kwargs: None,
            }))
        })
    }

    /// Disconnect from router gracefully 
    pub fn shutdown(&mut self) -> Pin<Box<dyn Future<Output = Result<(), CallError>>>> {
        let mut info = self.connection_info.lock().unwrap();

        if info.connection_state == ConnectionState::Connected {
            info.connection_state = ConnectionState::ShuttingDown;

            let (complete, receiver) = oneshot::channel();

            info.shutdown_complete = Some(complete);

            // TODO add timeout in case server doesn't respond.
            info.send_message(Message::Goodbye(
                ErrorDetails::new(),
                Reason::SystemShutdown,
            ))
            .unwrap();

            Box::pin(async {
                receiver.await.unwrap_or(Err(CallError {
                    reason: Reason::InternalError,
                    args: None,
                    kwargs: None,
                }))
            })
        } else {
            Box::pin(async {
                // Err(Error::new(ErrorKind::InvalidState(
                //     "Tried to shut down a client that was already shutting down",
                // )))
                Err(CallError {
                    reason: Reason::InternalError,
                    args: None,
                    kwargs: None,
                })
            })
        }
    }
}

impl fmt::Debug for ConnectionHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{Connection id: {}}}",
            self.connection_info.lock().unwrap().session_id
        )
    }
}
