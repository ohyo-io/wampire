use std::collections::HashMap;
use std::marker::Sync;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use log::{debug, info, trace};
use rand::distributions::{Distribution, Range};
use rand::thread_rng;
use ws::{listen as ws_listen, Result as WSResult, Sender};

use crate::messages::{ErrorDetails, Message, Reason};

use super::ID;

mod handshake;
mod messaging;
mod pubsub;
mod rpc;
use self::messaging::send_message;
use self::pubsub::SubscriptionPatternNode;
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
    let between = Range::new(0, 1u64.rotate_left(56) - 1);
    between.sample(&mut rng)
}

unsafe impl Sync for Router {}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    #[inline]
    pub fn new() -> Router {
        Router {
            info: Arc::new(RouterInfo {
                realms: Mutex::new(HashMap::new()),
            }),
        }
    }

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
