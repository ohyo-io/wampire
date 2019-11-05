use std::sync::Arc;

use log::{debug, info};

use crate::messages::{
    ErrorType, EventDetails, Message, PublishOptions, Reason, SubscribeOptions, URI,
};
use crate::{Dict, Error, ErrorKind, List, MatchingPolicy, WampResult};

use super::messaging::send_message;
use super::{random_id, ConnectionHandler};

mod patterns;
pub use self::patterns::SubscriptionPatternNode;

impl ConnectionHandler {
    pub fn handle_subscribe(
        &mut self,
        request_id: u64,
        options: SubscribeOptions,
        topic: URI,
    ) -> WampResult<()> {
        debug!(
            "Responding to subscribe message (id: {}, topic: {})",
            request_id, topic.uri
        );
        match self.realm {
            Some(ref realm) => {
                let mut realm = realm.lock().unwrap();
                let manager = &mut realm.subscription_manager;
                let topic_id = {
                    let topic_id = match manager.subscriptions.subscribe_with(
                        &topic,
                        Arc::clone(&self.info),
                        options.pattern_match,
                    ) {
                        Ok(topic_id) => topic_id,
                        Err(e) => {
                            return Err(Error::new(ErrorKind::ErrorReason(
                                ErrorType::Subscribe,
                                request_id,
                                e.reason(),
                            )))
                        }
                    };
                    self.subscribed_topics.push(topic_id);
                    topic_id
                };
                manager.subscription_ids_to_uris.insert(
                    topic_id,
                    (topic.uri, options.pattern_match == MatchingPolicy::Prefix),
                );
                send_message(&self.info, &Message::Subscribed(request_id, topic_id))
            }
            None => Err(Error::new(ErrorKind::InvalidState(
                "Received a message while not attached to a realm",
            ))),
        }
    }

    pub fn handle_unsubscribe(&mut self, request_id: u64, topic_id: u64) -> WampResult<()> {
        match self.realm {
            Some(ref realm) => {
                let mut realm = realm.lock().unwrap();
                let manager = &mut realm.subscription_manager;
                let (topic_uri, is_prefix) = match manager.subscription_ids_to_uris.get(&topic_id) {
                    Some(&(ref uri, ref is_prefix)) => (uri.clone(), *is_prefix),
                    None => {
                        return Err(Error::new(ErrorKind::ErrorReason(
                            ErrorType::Unsubscribe,
                            request_id,
                            Reason::NoSuchSubscription,
                        )))
                    }
                };

                let topic_id = match manager
                    .subscriptions
                    .unsubscribe_with(&topic_uri, &self.info, is_prefix)
                {
                    Ok(topic_id) => topic_id,
                    Err(e) => {
                        return Err(Error::new(ErrorKind::ErrorReason(
                            ErrorType::Unsubscribe,
                            request_id,
                            e.reason(),
                        )))
                    }
                };
                self.subscribed_topics.retain(|id| *id != topic_id);
                send_message(&self.info, &Message::Unsubscribed(request_id))
            }
            None => Err(Error::new(ErrorKind::InvalidState(
                "Received a message while not attached to a realm",
            ))),
        }
    }

    pub fn handle_publish(
        &mut self,
        request_id: u64,
        options: PublishOptions,
        topic: URI,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) -> WampResult<()> {
        debug!(
            "Responding to publish message (id: {}, topic: {})",
            request_id, topic.uri
        );
        match self.realm {
            Some(ref realm) => {
                let realm = realm.lock().unwrap();
                let manager = &realm.subscription_manager;
                let publication_id = random_id();
                let mut event_message =
                    Message::Event(1, publication_id, EventDetails::new(), args, kwargs);
                let my_id = { self.info.lock().unwrap().id };
                info!("Current topic tree: {:?}", manager.subscriptions);
                for (subscriber, topic_id, policy) in manager.subscriptions.filter(topic.clone()) {
                    if subscriber.lock().unwrap().id != my_id {
                        if let Message::Event(
                            ref mut old_topic,
                            ref _publish_id,
                            ref mut details,
                            ref _args,
                            ref _kwargs,
                        ) = event_message
                        {
                            *old_topic = topic_id;
                            details.topic = if policy == MatchingPolicy::Strict {
                                None
                            } else {
                                Some(topic.clone())
                            };
                        }
                        send_message(subscriber, &event_message)?;
                    }
                }
                if options.should_acknowledge() {
                    send_message(&self.info, &Message::Published(request_id, publication_id))?;
                }
                Ok(())
            }
            None => Err(Error::new(ErrorKind::InvalidState(
                "Received a message while not attached to a realm",
            ))),
        }
    }
}
