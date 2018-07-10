use ws::{connect, CloseCode, Error as WSError, ErrorKind as WSErrorKind, Handler, Handshake,
         Message as WSMessage, Request, Result as WSResult, Sender};

use ws::util::Token;

use eventual::{Complete, Future};
use messages::{CallOptions, ClientRoles, Dict, ErrorDetails, ErrorType, HelloDetails,
               InvocationDetails, List, MatchingPolicy, Message, PublishOptions, Reason,
               RegisterOptions, ResultDetails, SubscribeOptions, WelcomeDetails, YieldOptions, URI};
use rmp_serde::Deserializer as RMPDeserializer;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;
use std::sync::mpsc::{channel, Sender as CHSender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;
use url::Url;
use utils::StructMapWriter;
use {CallError, CallResult, Error, ErrorKind, WampResult, ID};

const CONNECTION_TIMEOUT: Token = Token(124);

pub struct Connection {
    // sender: Sender,
    // receiver: client::Receiver<stream::WebSocketStream>,
    realm: URI,
    url: String,
}

pub struct Subscription {
    pub topic: URI,
    subscription_id: ID,
}

pub struct Registration {
    pub procedure: URI,
    registration_id: ID,
}

struct SubscriptionCallbackWrapper {
    callback: Box<FnMut(List, Dict)>,
}

struct RegistrationCallbackWrapper {
    callback: Callback,
}

pub type Callback = Box<FnMut(List, Dict) -> CallResult<(Option<List>, Option<Dict>)>>;

static WAMP_JSON: &'static str = "wamp.2.json";
static WAMP_MSGPACK: &'static str = "wamp.2.msgpack";

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

pub struct Client {
    connection_info: Arc<Mutex<ConnectionInfo>>,
    max_session_id: ID,
}

pub struct ConnectionHandler {
    connection_info: Arc<Mutex<ConnectionInfo>>,
    realm: URI,
    state_transmission: CHSender<ConnectionResult>,
}

struct ConnectionInfo {
    connection_state: ConnectionState,
    sender: Sender,
    subscription_requests: HashMap<
        ID,
        (
            Complete<Subscription, CallError>,
            SubscriptionCallbackWrapper,
            URI,
        ),
    >,
    unsubscription_requests: HashMap<ID, (Complete<(), CallError>, ID)>,
    subscriptions: HashMap<ID, SubscriptionCallbackWrapper>,
    registrations: HashMap<ID, RegistrationCallbackWrapper>,
    call_requests: HashMap<ID, Complete<(List, Dict), CallError>>,
    registration_requests: HashMap<
        ID,
        (
            Complete<Registration, CallError>,
            RegistrationCallbackWrapper,
            URI,
        ),
    >,
    unregistration_requests: HashMap<ID, (Complete<(), CallError>, ID)>,
    protocol: String,
    publish_requests: HashMap<ID, Complete<ID, CallError>>,
    shutdown_complete: Option<Complete<(), CallError>>,
    session_id: ID,
}

trait MessageSender {
    fn send_message(&self, message: Message) -> WampResult<()>;
}

impl MessageSender for ConnectionInfo {
    fn send_message(&self, message: Message) -> WampResult<()> {
        debug!("Sending message {:?} via {}", message, self.protocol);
        let send_result = if self.protocol == WAMP_JSON {
            send_message_json(&self.sender, &message)
        } else {
            send_message_msgpack(&self.sender, &message)
        };
        match send_result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::new(ErrorKind::WSError(e))),
        }
    }
}

fn send_message_json(sender: &Sender, message: &Message) -> WSResult<()> {
    // Send the message
    sender.send(WSMessage::Text(serde_json::to_string(message).unwrap()))
}

fn send_message_msgpack(sender: &Sender, message: &Message) -> WSResult<()> {
    // Send the message
    let mut buf: Vec<u8> = Vec::new();
    message
        .serialize(&mut Serializer::with(&mut buf, StructMapWriter))
        .unwrap();
    sender.send(WSMessage::Binary(buf))
}

impl Connection {
    pub fn new(url: &str, realm: &str) -> Connection {
        Connection {
            realm: URI::new(realm),
            url: url.to_string(),
        }
    }

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
                    subscription_requests: HashMap::new(),
                    unsubscription_requests: HashMap::new(),
                    subscriptions: HashMap::new(),
                    registrations: HashMap::new(),
                    call_requests: HashMap::new(),
                    registration_requests: HashMap::new(),
                    unregistration_requests: HashMap::new(),
                    sender: out,
                    connection_state: ConnectionState::Connecting,
                    publish_requests: HashMap::new(),
                    shutdown_complete: None,
                    session_id: 0,
                }));

                ConnectionHandler {
                    state_transmission: tx.clone(),
                    connection_info: info,
                    realm: realm.clone(),
                }
            }).map_err(|e| Error::new(ErrorKind::WSError(e)));
            debug!("Result of connection: {:?}", connect_result);
            match connect_result {
                Ok(_) => (),
                Err(e) => {
                    tx.send(Err(e)).unwrap();
                }
            }
        });
        let info = try!(rx.recv().unwrap());
        Ok(Client {
            connection_info: info,
            max_session_id: 0,
        })
    }
}

macro_rules! cancel_future_tuple {
    ($dict:expr) => {{
        for (_, future) in $dict.drain() {
            future
                .0
                .fail(CallError::new(Reason::NetworkFailure, None, None));
        }
    }};
}

macro_rules! cancel_future {
    ($dict:expr) => {{
        for (_, future) in $dict.drain() {
            future.fail(CallError::new(Reason::NetworkFailure, None, None));
        }
    }};
}

impl Handler for ConnectionHandler {
    fn on_open(&mut self, handshake: Handshake) -> WSResult<()> {
        debug!("Connection Opened");
        let mut info = self.connection_info.lock().unwrap();
        info.protocol = match try!(handshake.response.protocol()) {
            Some(protocol) => protocol.to_string(),
            None => {
                warn!("Router did not specify protocol. Defaulting to wamp.2.json");
                WAMP_JSON.to_string()
            }
        };

        let hello_message =
            Message::Hello(self.realm.clone(), HelloDetails::new(ClientRoles::new()));
        debug!("Sending Hello message");
        thread::sleep(Duration::from_millis(200));
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
            promise.complete(());
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
        let mut request = try!(Request::from_url(url));
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
                    Message::Invocation(request_id, registration_id, details, args, kwargs) => {
                        self.handle_invocation(
                            info,
                            request_id,
                            registration_id,
                            details,
                            args,
                            kwargs,
                        )
                    }
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
                        promise.complete(())
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
        mut info: MutexGuard<ConnectionInfo>,
        request_id: ID,
        subscription_id: ID,
    ) {
        // TODO handle errors here
        info!("Received a subscribed notification");
        match info.subscription_requests.remove(&request_id) {
            Some((promise, callback, topic)) => {
                debug!("Completing promise");
                let subscription = Subscription {
                    topic: topic,
                    subscription_id: subscription_id,
                };
                info.subscriptions.insert(subscription_id, callback);
                drop(info);
                promise.complete(subscription)
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
        mut info: MutexGuard<ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        warn!("Received an error for a subscription");
        match info.subscription_requests.remove(&request_id) {
            Some((promise, _, _)) => {
                drop(info);
                promise.fail(CallError::new(reason, args, kwargs));
            }
            None => {
                warn!(
                    "Received a an error notification for a request we didn't make.  ID: {}",
                    request_id
                );
            }
        }
    }

    fn handle_unsubscribed(&self, mut info: MutexGuard<ConnectionInfo>, request_id: ID) {
        match info.unsubscription_requests.remove(&request_id) {
            Some((promise, subscription_id)) => {
                info.unsubscription_requests.remove(&subscription_id);
                drop(info);
                promise.complete(())
            }
            None => {
                warn!("Received a unsubscribed notification for a subscription we don't have.  ID: {}", request_id);
            }
        }
    }

    fn handle_unsubscribe_error(
        &self,
        mut info: MutexGuard<ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        match info.unsubscription_requests.remove(&request_id) {
            Some((promise, subscription_id)) => {
                info.unsubscription_requests.remove(&subscription_id);
                drop(info);
                promise.fail(CallError::new(reason, args, kwargs))
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
        mut info: MutexGuard<ConnectionInfo>,
        request_id: ID,
        registration_id: ID,
    ) {
        // TODO handle errors here
        info!("Received a registered notification");
        match info.registration_requests.remove(&request_id) {
            Some((promise, callback, procedure)) => {
                info.registrations.insert(registration_id, callback);
                drop(info);
                let registration = Registration {
                    procedure: procedure,
                    registration_id: registration_id,
                };
                promise.complete(registration)
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
        mut info: MutexGuard<ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        info!("Received a registration error");
        match info.registration_requests.remove(&request_id) {
            Some((promise, _, _)) => {
                drop(info);
                promise.fail(CallError::new(reason, args, kwargs))
            }
            None => {
                warn!(
                    "Received a registered error for a registration we don't have.  ID: {}",
                    request_id
                );
            }
        }
    }

    fn handle_unregistered(&self, mut info: MutexGuard<ConnectionInfo>, request_id: ID) {
        match info.unregistration_requests.remove(&request_id) {
            Some((promise, registration_id)) => {
                info.registrations.remove(&registration_id);
                drop(info);
                promise.complete(())
            }
            None => {
                warn!("Received a unregistered notification for a registration we don't have.  ID: {}", request_id);
            }
        }
    }

    fn handle_unregister_error(
        &self,
        mut info: MutexGuard<ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        match info.unregistration_requests.remove(&request_id) {
            Some((promise, _)) => {
                drop(info);
                promise.fail(CallError::new(reason, args, kwargs))
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
        mut info: MutexGuard<ConnectionInfo>,
        request_id: ID,
        publication_id: ID,
    ) {
        match info.publish_requests.remove(&request_id) {
            Some(promise) => {
                promise.complete(publication_id);
            }
            None => warn!(
                "Received published notification for a request we weren't tracking: {}",
                request_id
            ),
        }
    }
    fn handle_publish_error(
        &self,
        mut info: MutexGuard<ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        match info.publish_requests.remove(&request_id) {
            Some(promise) => promise.fail(CallError::new(reason, args, kwargs)),
            None => warn!("Received published error for a publication: {}", request_id),
        }
    }

    fn handle_welcome(
        &self,
        mut info: MutexGuard<ConnectionInfo>,
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

    fn handle_abort(&self, mut info: MutexGuard<ConnectionInfo>, reason: Reason) {
        error!("Router aborted connection.  Reason: {:?}", reason);
        info.connection_state = ConnectionState::ShuttingDown;
    }

    fn handle_event(
        &self,
        mut info: MutexGuard<ConnectionInfo>,
        subscription_id: ID,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        let args = args.unwrap_or_default();
        let kwargs = kwargs.unwrap_or_default();
        match info.subscriptions.get_mut(&subscription_id) {
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
        mut info: MutexGuard<ConnectionInfo>,
        request_id: ID,
        registration_id: ID,
        _details: InvocationDetails,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        let args = args.unwrap_or_default();
        let kwargs = kwargs.unwrap_or_default();
        let message = match info.registrations.get_mut(&registration_id) {
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
        mut info: MutexGuard<ConnectionInfo>,
        call_id: ID,
        _details: ResultDetails,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        let args = args.unwrap_or_default();
        let kwargs = kwargs.unwrap_or_default();
        match info.call_requests.remove(&call_id) {
            Some(promise) => {
                promise.complete((args, kwargs));
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
        mut info: MutexGuard<ConnectionInfo>,
        request_id: ID,
        reason: Reason,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) {
        match info.call_requests.remove(&request_id) {
            Some(promise) => promise.fail(CallError::new(reason, args, kwargs)),
            None => {
                warn!(
                    "Received an error for a call we didn't make.  ID: {}",
                    request_id
                );
            }
        }
    }

    fn handle_goodbye(&self, mut info: MutexGuard<ConnectionInfo>, reason: Reason) {
        info!("Router said goodbye.  Reason: {:?}", reason);

        info.send_message(Message::Goodbye(ErrorDetails::new(), Reason::GoodbyeAndOut))
            .unwrap();
        info.connection_state = ConnectionState::ShuttingDown;
    }

    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
    fn handle_error(
        &self,
        info: MutexGuard<ConnectionInfo>,
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
            },
            ErrorType::Unsubscribe => {
                self.handle_unsubscribe_error(info, request_id, reason, args, kwargs)
            },
            ErrorType::Publish => {
                self.handle_publish_error(info, request_id, reason, args, kwargs)
            },
            ErrorType::Register => {
                self.handle_register_error(info, request_id, reason, args, kwargs)
            },
            ErrorType::Unregister => {
                self.handle_unregister_error(info, request_id, reason, args, kwargs)
            },
            ErrorType::Invocation => {
                warn!("Received an error for an invocation message, which we did not (and could not) send")
            },
            ErrorType::Call => {
                self.handle_call_error(info, request_id, reason, args, kwargs)
            }
        }
    }
}

impl Client {
    fn get_next_session_id(&mut self) -> ID {
        self.max_session_id += 1;
        self.max_session_id
    }

    pub fn subscribe_with_pattern(
        &mut self,
        topic_pattern: URI,
        callback: Box<FnMut(List, Dict)>,
        policy: MatchingPolicy,
    ) -> WampResult<Future<Subscription, CallError>> {
        // Send a subscribe messages
        let request_id = self.get_next_session_id();
        let (complete, future) = Future::<Subscription, CallError>::pair();
        let callback = SubscriptionCallbackWrapper { callback: callback };
        let mut options = SubscribeOptions::new();
        if policy != MatchingPolicy::Strict {
            options.pattern_match = policy
        }
        let mut info = self.connection_info.lock().unwrap();
        info.subscription_requests
            .insert(request_id, (complete, callback, topic_pattern.clone()));
        try!(info.send_message(Message::Subscribe(request_id, options, topic_pattern)));
        Ok(future)
    }

    pub fn subscribe(
        &mut self,
        topic: URI,
        callback: Box<FnMut(List, Dict)>,
    ) -> WampResult<Future<Subscription, CallError>> {
        self.subscribe_with_pattern(topic, callback, MatchingPolicy::Strict)
    }

    pub fn register_with_pattern(
        &mut self,
        procedure_pattern: URI,
        callback: Callback,
        policy: MatchingPolicy,
    ) -> WampResult<Future<Registration, CallError>> {
        // Send a register message
        let request_id = self.get_next_session_id();
        let (complete, future) = Future::<Registration, CallError>::pair();
        let callback = RegistrationCallbackWrapper { callback: callback };
        let mut options = RegisterOptions::new();
        if policy != MatchingPolicy::Strict {
            options.pattern_match = policy
        }
        debug!("Acquiring lock on connection info");
        let mut info = self.connection_info.lock().unwrap();
        debug!("Lock on connection info acquired");
        info.registration_requests
            .insert(request_id, (complete, callback, procedure_pattern.clone()));
        try!(info.send_message(Message::Register(request_id, options, procedure_pattern)));
        Ok(future)
    }

    pub fn register(
        &mut self,
        procedure: URI,
        callback: Callback,
    ) -> WampResult<Future<Registration, CallError>> {
        self.register_with_pattern(procedure, callback, MatchingPolicy::Strict)
    }

    pub fn unsubscribe(&mut self, subscription: Subscription) -> WampResult<Future<(), CallError>> {
        let request_id = self.get_next_session_id();
        let mut info = self.connection_info.lock().unwrap();
        try!(info.send_message(Message::Unsubscribe(
            request_id,
            subscription.subscription_id
        )));
        let (complete, future) = Future::<(), CallError>::pair();
        info.unsubscription_requests
            .insert(request_id, (complete, subscription.subscription_id));
        Ok(future)
    }

    pub fn unregister(&mut self, registration: Registration) -> WampResult<Future<(), CallError>> {
        let request_id = self.get_next_session_id();
        let mut info = self.connection_info.lock().unwrap();
        try!(info.send_message(Message::Unregister(
            request_id,
            registration.registration_id
        )));
        let (complete, future) = Future::<(), CallError>::pair();

        info.unregistration_requests
            .insert(request_id, (complete, registration.registration_id));
        Ok(future)
    }

    pub fn publish(
        &mut self,
        topic: URI,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) -> WampResult<()> {
        info!("Publishing to {:?} with {:?} | {:?}", topic, args, kwargs);
        let request_id = self.get_next_session_id();
        self.connection_info
            .lock()
            .unwrap()
            .send_message(Message::Publish(
                request_id,
                PublishOptions::new(false),
                topic,
                args,
                kwargs,
            ))
    }

    pub fn call(
        &mut self,
        procedure: URI,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) -> WampResult<Future<(List, Dict), CallError>> {
        info!("Calling {:?} with {:?} | {:?}", procedure, args, kwargs);
        let request_id = self.get_next_session_id();
        let (complete, future) = Future::<(List, Dict), CallError>::pair();
        let mut info = self.connection_info.lock().unwrap();
        info.call_requests.insert(request_id, complete);
        try!(info.send_message(Message::Call(
            request_id,
            CallOptions::new(),
            procedure,
            args,
            kwargs
        )));
        Ok(future)
    }

    pub fn publish_and_acknowledge(
        &mut self,
        topic: URI,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) -> WampResult<Future<ID, CallError>> {
        info!("Publishing to {:?} with {:?} | {:?}", topic, args, kwargs);
        let request_id = self.get_next_session_id();
        let (complete, future) = Future::<ID, CallError>::pair();
        let mut info = self.connection_info.lock().unwrap();
        info.publish_requests.insert(request_id, complete);
        try!(info.send_message(Message::Publish(
            request_id,
            PublishOptions::new(true),
            topic,
            args,
            kwargs
        )));
        Ok(future)
    }

    pub fn shutdown(&mut self) -> WampResult<Future<(), CallError>> {
        let mut info = self.connection_info.lock().unwrap();
        if info.connection_state == ConnectionState::Connected {
            info.connection_state = ConnectionState::ShuttingDown;
            let (complete, future) = Future::pair();
            info.shutdown_complete = Some(complete);
            // TODO add timeout in case server doesn't respond.
            try!(info.send_message(Message::Goodbye(
                ErrorDetails::new(),
                Reason::SystemShutdown
            )));
            Ok(future)
        } else {
            Err(Error::new(ErrorKind::InvalidState(
                "Tried to shut down a client that was already shutting down",
            )))
        }
    }
}

impl fmt::Debug for ConnectionHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{Connection id: {}}}",
            self.connection_info.lock().unwrap().session_id
        )
    }
}
