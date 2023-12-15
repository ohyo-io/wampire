//! # Message Routing in WAMP[](#message-routing-in-wamp "Permalink to this headline")
//!
//! - [Loosely coupled](#loosely-coupled)
//! - [Component based](#component-based)
//! - [Real-time](#real-time)
//! - [Language independent](#language-independent)
//! - [Network spanning](#network-spanning)
//!
//!
//! ---
//!
//! WAMP provides [Unified Application Routing](#unified-application-routing) in an open [WebSocket protocol](#websocket-protocol) 
//! that works with [different](#different) languages.
//!
//! Using WAMP you can build distributed systems out of application components which are **loosely coupled** 
//! and communicate in (soft) **real-time**.
//!
//! At its core, WAMP offers two communication patterns for application components to talk to each other:
//!
//! - [Publish & Subscribe](https://wamp-proto.org/faq.html#pubsub) (PubSub)
//! - [Remote Procedure Calls](https://wamp-proto.org/faq.html#rpc) (RPC)
//!
//! We think applications often have a natural [need for both forms of communication](https://wamp-proto.org/faq.html#why_rpc_and_pubsub) 
//! and shouldn’t be required to use different protocols/means for those. Which is why WAMP provides both.
//!
//! WAMP is easy to use, simple to implement and based on modern Web standards: WebSocket, JSON and URIs.
//!
//! While WAMP isn’t exactly rocket science, we believe it’s good engineering and a major step forward in practice 
//! that allows developers to create more powerful applications with less complexity and in less time.
//!
//! ## [Loosely coupled](#id6)[](#loosely-coupled "Permalink to this headline")
//!
//! WAMP provides what we call **unified Application Routing** for application communication:
//!
//! - routing of events in the Publish & Subscriber pattern and
//! - routing of calls in the Remote Procedure Call pattern
//!
//! between applications components in *one* protocol.
//!
//! Unified routing is probably best explained by contrasting it with legacy approaches.
//!
//! Lets take the old “client-server” world. In the client-server model, a remote procedure call goes 
//! directly from the *Caller* to the *Callee*:
//!
//! ![](https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/unified_routing_rpc_client_server.svg)
//!
//! Remote procedure calls in the **Client-Server** model[](#id1 "Permalink to this image")
//!
//! In the client-server model, a *Caller* needs to have knowledge about where the *Callee* resides and how to reach it. 
//! This introduces a strong coupling between *Caller* and *Callee*. Which is bad, because applications can quickly 
//! become complex and unmaintainable. We explain how WAMP fixes that in a minute.
//!
//! The problems coming from strong coupling between application components were long recognized and this (besides other requirements) 
//! lead to the publish-subscribe model.
//!
//! In the publish-subscribe model a *Publisher* submits information to an abstract “topic”, and *Subscribers* only receive 
//! information indirectly by announcing their interest on a respective “topic”. Both do not know about each other. 
//! They are decoupled via the “topic” and via an intermediary usually called *Broker*:
//!
//! ![](https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/unified_routing_pubsub_broker.svg)
//!
//! A Broker decouples *Publishers* and *Subscribers*[](#id2 "Permalink to this image")
//!
//! A *Broker* keeps a book of subscriptions: who is currently subscribed on which topic. When a *Publisher* publishes 
//! some information (“event”) to a topic, the *Broker* will look up who is currently subscribed on that topic: 
//! determine the set of *Subscribers* on the topic published to. And then forward the information (“event”) to all those *Subscribers*.
//!
//! The act of determining receivers of information (independently of the information submitted) and forwarding 
//! the information to receivers is called *routing*.
//!
//! Now, WAMP translates the benefits of loose coupling to RPC. Different from the client-server model, WAMP also 
//! decouples *Callers* and *Callees* by introducing an intermediary - the *Dealer*:
//!
//! ![](https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/unified_routing_rpc_dealer.svg)
//!
//! Remote procedure calls in the **Dealer** model[](#id3 "Permalink to this image")
//!
//! Similar to a *Broker’s* role with PubSub, the *Dealer* is responsible for routing a call originating 
//! from the *Caller* to the *Callee* and route back results or errors vice-versa. Both do not know about each other: 
//! where the peer resides and how to reach it. This knowledge is encapsulated in the *Dealer*
//!
//! With WAMP, a *Callee* registers a procedure at a *Dealer* under an abstract name: a URI identifying the procedure. 
//! When a *Caller* wants to call a remote procedure, it talks to the *Dealer* and only provides the URI of the procedure 
//! to be called plus any call arguments. The *Dealer* will look up the procedure to be invoked in his book of registered procedures. 
//! The information from the book includes *where* the *Callee* implementing the procedure resides, and how to reach it.
//!
//! In effect, *Callers* and *Callees* are decoupled, and applications can use RPC and still benefit from loose coupling.
//!
//! ## [Component based](#id7)[](#component-based "Permalink to this headline")
//!
//! **Brokers, Dealers and Routers**
//!
//! What if you combine a Broker (for Publish & Subscribe) and a Dealer (for routed Remote Procedure Calls)?
//!
//! When you combine a *Broker* and a *Dealer* you get what WAMP calls a *Router*:
//!
//! ![](https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/unified_routing_broker_dealer.svg)
//!
//! A **Router** combines a Broker and a Dealer[](#id4 "Permalink to this image")
//!
//! A *Router* is capable of routing both calls and events, and hence can support flexible, decoupled architectures 
//! that use both RPC and PubSub. We think this is new. And a good thing.
//!
//! Here is an example. Imagine you have a small embedded device like an Arduino Yun with sensors (like a temperature sensor) 
//! and actuators (like a light or motor) connected. And you want to integrate the device into an overall system with user 
//! facing frontend to control the actuators, and continuously process sensor values in a backend component.
//!
//! Using WAMP, you can have a browser-based UI, the embedded device and your backend talk to each other in real-time:
//!
//! ![](https://raw.githubusercontent.com/wiki/ohyo-io/wampire/images/unified_routing_wamp_iot.svg)
//!
//! WAMP in an IoT application[](#id5 "Permalink to this image")
//!
//! Switching on a light on the device from the browser-based UI is naturally done by calling a remote procedure on the device (1). 
//! And the sensor values generated by the device continuously are naturally transmitted to the backend 
//! component (and possibly others) via publish & subscribe (2).
//!
//! > “Moving onto the part of Internet of Things, we integrated a sensor (light sensor) and an actuator (light switch/dimmer) 
//! > into a web application. The major feature of the sensor (sending data) and that of the actuator (commanding and configuration)
//! > perfectly match the messaging patterns, Pub/Sub and RPC, which WAMP provides.”
//!
//! From [Web Technologies for the Internet of Things](https://into.aalto.fi/download/attachments/12324178/Huang_Fuguo_thesis_2.pdf), 
//! Master thesis, July 2013, Huang F.
//!
//! **So here you have it: one protocol fulfilling “all” application communication needs.**
//!
//! ## [Real-time](#id8)[](#real-time "Permalink to this headline")
//!
//! [WebSocket](http://crossbario.com/blog/Websocket-Why-What-Can-I-Use-It/) is a new Web protocol that overcomes limitations 
//! of HTTP when bidirectional, real-time communication is required.
//!
//! WebSocket is specified as an [IETF standard](http://tools.ietf.org/html/rfc6455) and built into [modern browsers](https://caniuse.com/#search=websocket).
//!
//! When designing WAMP, we recognized early on that WebSocket would be the ideal basis for WAMP as it provides bidirectional 
//! real-time messaging that is compatible with the Web and browsers. Not only that - we can run WebSocket with non-browser environments as well.
//!
//! However, as such, WebSocket it is quite low-level and only provides raw messaging. This is where WAMP enters. 
//! WAMP adds the higher level messaging patterns of RPC and PubSub to WebSocket.
//!
//! Technically, WAMP is an [officially registered](http://www.iana.org/assignments/websocket/websocket.xml#subprotocol-name) 
//! **WebSocket subprotocol** (runs on top of WebSocket) that uses [JSON](http://www.json.org/) as message serialization format.
//!
//! While WAMP-over-WebSocket with JSON serialization is the preferred transport for WAMP, the protocol can also run with 
//! [MsgPack](http://msgpack.org/) as serialization, run over raw-TCP or generally any message based, bidirectional, reliable transport.
//!
//! **Hence: WAMP runs on the Web and anywhere else.**
//!
//! ## [Language independent](#id9)[](#language-independent "Permalink to this headline")
//!
//! WAMP was designed with first-class support for [different languages](https://wamp-proto.org/implementations.html) 
//! in mind (*). Nothing in WAMP is specific to a single programming language. As soon as a programming language has a WAMP implementation, 
//! it can talk to application components written *in any other language* with WAMP support. Transparently.
//!
//! > WAMP has facilities for first-class support of many common and less common language features. E.g. WAMP can transmit both positional 
//! > and keyword based call arguments, so that languages which natively support keyword arguments in functions (e.g. Python) can be naturally mapped. 
//! > WAMP even supports multi-positional and keywords based **return** values for calls. E.g. the PostgreSQL pgPL/SQL or Oracle PL/SQL languages support this. 
//! > Means that most PL/SQL functions can be naturally exposed via WAMP.
//!
//! The ability to create a system from application components written in different languages is a big advantage. You can write your frontend 
//! in JavaScript to run in the browser, but still write backend components in Python or Java. If you recognize a performance bottleneck in a component, 
//! you can rewrite that component in a faster language - without changing a single line of code in other components.
//!
//! All developers in your team can become productive, since they are not tied to a “least common denominator”, but can write components in the 
//! language they prefer, or which is ideal for the specific components at hand. Need some fancy numerical code which is only available in C++ and 
//! needs to run with maximum performance? No problem. Have the functionality isolated in an application component written in C++, and integrate this 
//! with components written in your “standard” language.
//!
//! **What this means is: plug-and-play your app components - no matter what language.**

use std::{
    collections::HashMap,
    marker::Sync,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use log::{debug, info, trace};
use rand::{thread_rng, Rng};
use parity_ws::{listen as ws_listen, Result as WSResult, Sender};

use crate::messages::{ErrorDetails, Message, Reason};

use super::ID;

mod handshake;

mod messaging;
use self::messaging::send_message;

mod pubsub;
use self::pubsub::SubscriptionPatternNode;

mod rpc;
use self::rpc::RegistrationPatternNode;

struct SubscriptionManager {
    subscriptions: SubscriptionPatternNode<Arc<Mutex<ConnectionInfo>>>,
    subscription_ids_to_uris: HashMap<u64, (String, bool)>,
}

struct RegistrationManager {
    registrations: RegistrationPatternNode<Arc<Mutex<ConnectionInfo>>>,
    registration_ids_to_uris: HashMap<u64, (String, bool)>,
    active_calls: HashMap<ID, (ID, Arc<Mutex<ConnectionInfo>>)>,
}

struct Realm {
    subscription_manager: SubscriptionManager,
    registration_manager: RegistrationManager,
    connections: Vec<Arc<Mutex<ConnectionInfo>>>,
}

/// Represents WAMP Router
pub struct Router {
    info: Arc<RouterInfo>,
}

struct RouterInfo {
    realms: Mutex<HashMap<String, Arc<Mutex<Realm>>>>,
}

struct ConnectionHandler {
    info: Arc<Mutex<ConnectionInfo>>,
    router: Arc<RouterInfo>,
    realm: Option<Arc<Mutex<Realm>>>,
    subscribed_topics: Vec<ID>,
    registered_procedures: Vec<ID>,
}

/// Represents WAMP Router connection information
pub struct ConnectionInfo {
    state: ConnectionState,
    sender: Sender,
    protocol: String,
    id: u64,
}

#[derive(Clone, PartialEq)]
enum ConnectionState {
    Initializing,
    Connected,
    ShuttingDown,
    Disconnected,
}

static WAMP_JSON: &str = "wamp.2.json";
static WAMP_MSGPACK: &str = "wamp.2.msgpack";

fn random_id() -> u64 {
    let mut rng = thread_rng();
    // TODO make this a constant
    rng.gen_range(1..1u64.rotate_left(53))
}

unsafe impl Sync for Router {}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    /// Create the new default router
    #[inline]
    pub fn new() -> Router {
        Router {
            info: Arc::new(RouterInfo {
                realms: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Start listrning with url
    pub fn listen(&self, url: &str) -> JoinHandle<()> {
        let router_info = Arc::clone(&self.info);
        let url = url.to_string();
        thread::spawn(move || {
            ws_listen(&url[..], |sender| ConnectionHandler {
                info: Arc::new(Mutex::new(ConnectionInfo {
                    state: ConnectionState::Initializing,
                    sender,
                    protocol: String::new(),
                    id: random_id(),
                })),
                subscribed_topics: Vec::new(),
                registered_procedures: Vec::new(),
                realm: None,
                router: Arc::clone(&router_info),
            })
            .unwrap();
        })
    }

    /// Add realm to router
    pub fn add_realm(&mut self, realm: &str) {
        let mut realms = self.info.realms.lock().unwrap();
        if realms.contains_key(realm) {
            return;
        }
        realms.insert(
            realm.to_string(),
            Arc::new(Mutex::new(Realm {
                connections: Vec::new(),
                subscription_manager: SubscriptionManager {
                    subscriptions: SubscriptionPatternNode::new(),
                    subscription_ids_to_uris: HashMap::new(),
                },
                registration_manager: RegistrationManager {
                    registrations: RegistrationPatternNode::new(),
                    registration_ids_to_uris: HashMap::new(),
                    active_calls: HashMap::new(),
                },
            })),
        );
        debug!("Added realm {}", realm);
    }

    /// Shut down the router gracefully
    pub fn shutdown(&self) {
        for realm in self.info.realms.lock().unwrap().values() {
            for connection in &realm.lock().unwrap().connections {
                send_message(
                    connection,
                    &Message::Goodbye(ErrorDetails::new(), Reason::SystemShutdown),
                )
                .ok();
                let mut connection = connection.lock().unwrap();
                connection.state = ConnectionState::ShuttingDown;
            }
        }
        info!("Goodbye messages sent.  Waiting 5 seconds for response");
        thread::sleep(Duration::from_secs(5));
        for realm in self.info.realms.lock().unwrap().values() {
            for connection in &realm.lock().unwrap().connections {
                let connection = connection.lock().unwrap();
                connection.sender.shutdown().ok();
            }
        }
    }
}

impl ConnectionHandler {
    fn remove(&mut self) {
        if let Some(ref realm) = self.realm {
            let mut realm = realm.lock().unwrap();
            {
                trace!(
                    "Removing subscriptions for client {}",
                    self.info.lock().unwrap().id
                );
                let manager = &mut realm.subscription_manager;
                for subscription_id in &self.subscribed_topics {
                    trace!("Looking for subscription {}", subscription_id);
                    if let Some(&(ref topic_uri, is_prefix)) =
                        manager.subscription_ids_to_uris.get(subscription_id)
                    {
                        trace!("Removing subscription to {:?}", topic_uri);
                        manager
                            .subscriptions
                            .unsubscribe_with(topic_uri, &self.info, is_prefix)
                            .ok();
                        trace!("Subscription tree: {:?}", manager.subscriptions);
                    }
                }
            }
            {
                let manager = &mut realm.registration_manager;
                for registration_id in &self.registered_procedures {
                    if let Some(&(ref topic_uri, is_prefix)) =
                        manager.registration_ids_to_uris.get(registration_id)
                    {
                        manager
                            .registrations
                            .unregister_with(topic_uri, &self.info, is_prefix)
                            .ok();
                    }
                }
            }
            let my_id = self.info.lock().unwrap().id;
            realm
                .connections
                .retain(|connection| connection.lock().unwrap().id != my_id);
        }
    }

    fn terminate_connection(&mut self) -> WSResult<()> {
        self.remove();
        Ok(())
    }
}
