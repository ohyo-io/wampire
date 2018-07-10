use std::fmt;

pub use messages::types::*;
use serde;
use ID;
mod types;

macro_rules! try_or {
    ($e:expr, $msg:expr) => {
        match try!($e) {
            Some(val) => val,
            None => return Err(serde::de::Error::custom($msg)),
        }
    };
}

#[derive(Debug, PartialEq)]
pub enum Message {
    Hello(URI, HelloDetails),
    Welcome(ID, WelcomeDetails),
    Abort(ErrorDetails, Reason),
    Goodbye(ErrorDetails, Reason),
    Error(ErrorType, ID, Dict, Reason, Option<List>, Option<Dict>),
    Subscribe(ID, SubscribeOptions, URI),
    Subscribed(ID, ID),
    Unsubscribe(ID, ID),
    Unsubscribed(ID),
    Publish(ID, PublishOptions, URI, Option<List>, Option<Dict>),
    Published(ID, ID),
    Event(ID, ID, EventDetails, Option<List>, Option<Dict>),
    Register(ID, RegisterOptions, URI),
    Registered(ID, ID),
    Unregister(ID, ID),
    Unregistered(ID),
    Call(ID, CallOptions, URI, Option<List>, Option<Dict>),
    Invocation(ID, ID, InvocationDetails, Option<List>, Option<Dict>),
    Yield(ID, YieldOptions, Option<List>, Option<Dict>),
    Result(ID, ResultDetails, Option<List>, Option<Dict>),
}

macro_rules! serialize_with_args {
    ($args:expr, $kwargs:expr, $serializer:expr, $($item: expr),*) => (
        if let Some(ref kwargs) = *$kwargs {
                if let Some(ref args) =  *$args {
                    ( $($item,)* args, kwargs).serialize($serializer)
                } else {
                    ( $($item,)* Vec::<u8>::new(), kwargs).serialize($serializer)
                }
            } else {
                if let Some(ref args) = *$args {
                    ( $($item,)* args).serialize($serializer)
                } else {
                    ( $($item,)*).serialize($serializer)
                }

            }
    );
}

impl serde::Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Message::Hello(ref realm, ref details) => (1, &realm, details).serialize(serializer),
            Message::Welcome(ref session, ref details) => {
                (2, session, details).serialize(serializer)
            }
            Message::Abort(ref details, ref reason) => (3, details, reason).serialize(serializer),
            Message::Goodbye(ref details, ref reason) => (6, details, reason).serialize(serializer),
            Message::Error(ref ty, id, ref details, ref reason, ref args, ref kwargs) => {
                serialize_with_args!(args, kwargs, serializer, 8, ty, id, details, reason)
            }
            Message::Subscribe(request_id, ref options, ref topic) => {
                (32, request_id, options, topic).serialize(serializer)
            }
            Message::Subscribed(request_id, subscription_id) => {
                (33, request_id, subscription_id).serialize(serializer)
            }
            Message::Unsubscribe(request_id, subscription_id) => {
                (34, request_id, subscription_id).serialize(serializer)
            }
            Message::Unsubscribed(request_id) => (35, request_id).serialize(serializer),
            Message::Publish(id, ref details, ref topic, ref args, ref kwargs) => {
                serialize_with_args!(args, kwargs, serializer, 16, id, details, topic)
            }
            Message::Published(request_id, publication_id) => {
                (17, request_id, publication_id).serialize(serializer)
            }
            Message::Event(subscription_id, publication_id, ref details, ref args, ref kwargs) => {
                serialize_with_args!(
                    args,
                    kwargs,
                    serializer,
                    36,
                    subscription_id,
                    publication_id,
                    details
                )
            }
            Message::Register(request_id, ref options, ref procedure) => {
                (64, request_id, options, procedure).serialize(serializer)
            }
            Message::Registered(request_id, registration_id) => {
                (65, request_id, registration_id).serialize(serializer)
            }
            Message::Unregister(request_id, registration_id) => {
                (66, request_id, registration_id).serialize(serializer)
            }
            Message::Unregistered(request_id) => (67, request_id).serialize(serializer),
            Message::Call(id, ref options, ref topic, ref args, ref kwargs) => {
                serialize_with_args!(args, kwargs, serializer, 48, id, options, topic)
            }
            Message::Invocation(id, registration_id, ref details, ref args, ref kwargs) => {
                serialize_with_args!(args, kwargs, serializer, 68, id, registration_id, details)
            }
            Message::Yield(id, ref options, ref args, ref kwargs) => {
                serialize_with_args!(args, kwargs, serializer, 70, id, options)
            }
            Message::Result(id, ref details, ref args, ref kwargs) => {
                serialize_with_args!(args, kwargs, serializer, 50, id, details)
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Message, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(MessageVisitor)
    }
}

struct MessageVisitor;

impl MessageVisitor {
    fn visit_hello<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let uri = try_or!(
            visitor.next_element(),
            "Hello message ended before realm uri"
        );
        let details = try_or!(
            visitor.next_element(),
            "Hello message ended before details dict"
        );
        Ok(Message::Hello(uri, details))
    }

    fn visit_welcome<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let session = try_or!(
            visitor.next_element(),
            "Welcome message ended before session id"
        );
        let details = try_or!(
            visitor.next_element(),
            "Welcome message ended before details dict"
        );
        Ok(Message::Welcome(session, details))
    }

    fn visit_abort<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let details = try_or!(
            visitor.next_element(),
            "Abort message ended before details dict"
        );
        let reason = try_or!(
            visitor.next_element(),
            "Abort message ended before reason uri"
        );
        Ok(Message::Abort(details, reason))
    }

    fn visit_goodbye<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let details = try_or!(
            visitor.next_element(),
            "Goodbye message ended before details dict"
        );
        let reason = try_or!(
            visitor.next_element(),
            "Goodbye message ended before reason uri"
        );
        Ok(Message::Goodbye(details, reason))
    }

    fn visit_error<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let message_type = try_or!(
            visitor.next_element(),
            "Error message ended before message type"
        );
        let id = try_or!(
            visitor.next_element(),
            "Error message ended before session id"
        );
        let details = try_or!(
            visitor.next_element(),
            "Error message ended before details dict"
        );
        let reason = try_or!(
            visitor.next_element(),
            "Error message ended before reason uri"
        );
        let args = try!(visitor.next_element());
        let kwargs = try!(visitor.next_element());
        Ok(Message::Error(
            message_type,
            id,
            details,
            reason,
            args,
            kwargs,
        ))
    }

    fn visit_subscribe<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let request = try_or!(
            visitor.next_element(),
            "Subscribe message ended before request id"
        );
        let options = try_or!(
            visitor.next_element(),
            "Subscribe message ended before options dict"
        );
        let topic = try_or!(
            visitor.next_element(),
            "Subscribe message ended before topic uri"
        );
        Ok(Message::Subscribe(request, options, topic))
    }

    fn visit_subscribed<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let request = try_or!(
            visitor.next_element(),
            "Subscribed message ended before request id"
        );
        let subscription = try_or!(
            visitor.next_element(),
            "Subscribed message ended before subscription id"
        );
        Ok(Message::Subscribed(request, subscription))
    }

    fn visit_unsubscribe<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let request = try_or!(
            visitor.next_element(),
            "Unsubscribe message ended before request id"
        );
        let subscription = try_or!(
            visitor.next_element(),
            "Unsubscribe message ended before subscription id"
        );
        Ok(Message::Unsubscribe(request, subscription))
    }

    fn visit_unsubscribed<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let request = try_or!(
            visitor.next_element(),
            "Unsubscribed message ended before request id"
        );
        Ok(Message::Unsubscribed(request))
    }

    fn visit_publish<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let id = try_or!(
            visitor.next_element(),
            "Publish message ended before session id"
        );
        let details = try_or!(
            visitor.next_element(),
            "Publish message ended before details dict"
        );
        let topic = try_or!(
            visitor.next_element(),
            "Publish message ended before topic uri"
        );
        let args = try!(visitor.next_element());
        let kwargs = try!(visitor.next_element());
        Ok(Message::Publish(id, details, topic, args, kwargs))
    }

    fn visit_published<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let request = try_or!(
            visitor.next_element(),
            "Published message ended before request id"
        );
        let publication = try_or!(
            visitor.next_element(),
            "Published message ended before publication id"
        );
        Ok(Message::Published(request, publication))
    }

    fn visit_event<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let subscription_id = try_or!(
            visitor.next_element(),
            "Event message ended before session subscription id"
        );
        let publication_id = try_or!(
            visitor.next_element(),
            "Event message ended before publication id"
        );
        let details = try_or!(
            visitor.next_element(),
            "Event message ended before details dict"
        );
        let args = try!(visitor.next_element());
        let kwargs = try!(visitor.next_element());
        Ok(Message::Event(
            subscription_id,
            publication_id,
            details,
            args,
            kwargs,
        ))
    }

    fn visit_register<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let request = try_or!(
            visitor.next_element(),
            "Register message ended before request id"
        );
        let options = try_or!(
            visitor.next_element(),
            "Register message ended before request options"
        );
        let procedure = try_or!(
            visitor.next_element(),
            "Register message ended before procedure"
        );
        Ok(Message::Register(request, options, procedure))
    }

    fn visit_registered<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let request = try_or!(
            visitor.next_element(),
            "Registered message ended before request id"
        );
        let registration_id = try_or!(
            visitor.next_element(),
            "Registered message ended before registration id"
        );
        Ok(Message::Registered(request, registration_id))
    }

    fn visit_unregister<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let request = try_or!(
            visitor.next_element(),
            "Registered message ended before request id"
        );
        let registration_id = try_or!(
            visitor.next_element(),
            "Registered message ended before registration id"
        );
        Ok(Message::Unregister(request, registration_id))
    }

    fn visit_unregistered<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let request = try_or!(
            visitor.next_element(),
            "Registered message ended before request id"
        );
        Ok(Message::Unregistered(request))
    }

    fn visit_call<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let id = try_or!(
            visitor.next_element(),
            "Call message ended before session id"
        );
        let options = try_or!(
            visitor.next_element(),
            "Call message ended before options dict"
        );
        let topic = try_or!(
            visitor.next_element(),
            "Call message ended before procedure uri"
        );
        let args = try!(visitor.next_element());
        let kwargs = try!(visitor.next_element());
        Ok(Message::Call(id, options, topic, args, kwargs))
    }

    fn visit_invocation<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let id = try_or!(
            visitor.next_element(),
            "Invocation message ended before session id"
        );
        let registration_id = try_or!(
            visitor.next_element(),
            "Invocation message ended before registration id"
        );
        let details = try_or!(
            visitor.next_element(),
            "Invocation message ended before details dict"
        );
        let args = try!(visitor.next_element());
        let kwargs = try!(visitor.next_element());
        Ok(Message::Invocation(
            id,
            registration_id,
            details,
            args,
            kwargs,
        ))
    }

    fn visit_yield<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let id = try_or!(
            visitor.next_element(),
            "Yield message ended before session id"
        );
        let options = try_or!(
            visitor.next_element(),
            "Yield message ended before options dict"
        );
        let args = try!(visitor.next_element());
        let kwargs = try!(visitor.next_element());
        Ok(Message::Yield(id, options, args, kwargs))
    }

    fn visit_result<'de, V>(&self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let id = try_or!(
            visitor.next_element(),
            "Result message ended before session id"
        );
        let details = try_or!(
            visitor.next_element(),
            "Result message ended before details dict"
        );
        let args = try!(visitor.next_element());
        let kwargs = try!(visitor.next_element());
        Ok(Message::Result(id, details, args, kwargs))
    }
}

impl<'de> serde::de::Visitor<'de> for MessageVisitor {
    type Value = Message;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a message")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Message, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let message_type: u64 = try_or!(visitor.next_element(), "No message type found");
        match message_type {
            1 => self.visit_hello(visitor),
            2 => self.visit_welcome(visitor),
            3 => self.visit_abort(visitor),
            6 => self.visit_goodbye(visitor),
            8 => self.visit_error(visitor),
            32 => self.visit_subscribe(visitor),
            33 => self.visit_subscribed(visitor),
            34 => self.visit_unsubscribe(visitor),
            35 => self.visit_unsubscribed(visitor),
            16 => self.visit_publish(visitor),
            17 => self.visit_published(visitor),
            36 => self.visit_event(visitor),
            64 => self.visit_register(visitor),
            65 => self.visit_registered(visitor),
            66 => self.visit_unregister(visitor),
            67 => self.visit_unregistered(visitor),
            48 => self.visit_call(visitor),
            68 => self.visit_invocation(visitor),
            70 => self.visit_yield(visitor),
            50 => self.visit_result(visitor),
            _ => Err(serde::de::Error::custom("Unknown message type")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::types::{CallOptions, ClientRoles, ErrorDetails, ErrorType, EventDetails,
                       HelloDetails, InvocationDetails, PublishOptions, Reason, RegisterOptions,
                       ResultDetails, RouterRoles, SubscribeOptions, Value, WelcomeDetails,
                       YieldOptions, URI};
    use super::Message;
    use rmp_serde::Deserializer as RMPDeserializer;
    use rmp_serde::Serializer;
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::collections::HashMap;
    use utils::StructMapWriter;

    macro_rules! two_way_test {
        ($message:expr, $s:expr) => {{
            let message = $message;
            assert_eq!(serde_json::to_string(&message).unwrap(), $s);
            assert_eq!(serde_json::from_str::<Message>($s).unwrap(), message);
            let mut buf: Vec<u8> = Vec::new();
            message
                .serialize(&mut Serializer::with(&mut buf, StructMapWriter))
                .unwrap();
            let mut de = RMPDeserializer::new(&buf[..]);
            let new_message: Message = Deserialize::deserialize(&mut de).unwrap();
            assert_eq!(new_message, message);
        }};
    }

    #[test]
    fn serialize_hello() {
        two_way_test!(
            Message::Hello(URI::new("ca.dal.wamp.test"), HelloDetails::new(ClientRoles::new_basic())),
            "[1,\"ca.dal.wamp.test\",{\"roles\":{\"publisher\":{\"features\":{}},\"subscriber\":{\"features\":{}},\"caller\":{\"features\":{}},\"callee\":{\"features\":{}}}}]"
        );
        two_way_test!(
            Message::Hello(URI::new("ca.dal.wamp.test"), HelloDetails::new_with_agent(ClientRoles::new(), "dal_wamp")),
            "[1,\"ca.dal.wamp.test\",{\"agent\":\"dal_wamp\",\"roles\":{\"publisher\":{\"features\":{}},\"subscriber\":{\"features\":{\"pattern_based_subscription\":true}},\"caller\":{\"features\":{}},\"callee\":{\"features\":{}}}}]"
        )
    }

    #[test]
    fn serialize_welcome() {
        two_way_test!(
            Message::Welcome(493782, WelcomeDetails::new(RouterRoles::new_basic())),
            "[2,493782,{\"roles\":{\"dealer\":{},\"broker\":{}}}]"
        );
        two_way_test!(
            Message::Welcome(493782, WelcomeDetails::new_with_agent(RouterRoles::new(), "dal_wamp")),
            "[2,493782,{\"agent\":\"dal_wamp\",\"roles\":{\"dealer\":{\"features\":{\"pattern_based_registration\":true}},\"broker\":{\"features\":{\"pattern_based_subscription\":true}}}}]"
        );
    }

    #[test]
    fn serialize_abort() {
        two_way_test!(
            Message::Abort(ErrorDetails::new(), Reason::NoSuchRealm),
            "[3,{},\"wamp.error.no_such_realm\"]"
        );
        two_way_test!(
            Message::Abort(
                ErrorDetails::new_with_message("The realm does not exist"),
                Reason::NoSuchRealm
            ),
            "[3,{\"message\":\"The realm does not exist\"},\"wamp.error.no_such_realm\"]"
        );
    }

    #[test]
    fn serialize_goodbye() {
        two_way_test!(
            Message::Goodbye(ErrorDetails::new(), Reason::GoodbyeAndOut),
            "[6,{},\"wamp.error.goodbye_and_out\"]"
        );
        two_way_test!(
            Message::Goodbye(
                ErrorDetails::new_with_message("The host is shutting down now"),
                Reason::SystemShutdown
            ),
            "[6,{\"message\":\"The host is shutting down now\"},\"wamp.error.system_shutdown\"]"
        );
    }

    #[test]
    fn serialize_error() {
        two_way_test!(
            Message::Error(
                ErrorType::Subscribe,
                713845233,
                HashMap::new(),
                Reason::NotAuthorized,
                None,
                None
            ),
            "[8,32,713845233,{},\"wamp.error.not_authorized\"]"
        );

        two_way_test!(
            Message::Error(
                ErrorType::Unsubscribe,
                3746383,
                HashMap::new(),
                Reason::InvalidURI,
                Some(Vec::new()),
                None
            ),
            "[8,34,3746383,{},\"wamp.error.invalid_uri\",[]]"
        );

        two_way_test!(
            Message::Error(
                ErrorType::Register,
                8534533,
                HashMap::new(),
                Reason::InvalidArgument,
                Some(Vec::new()),
                Some(HashMap::new())
            ),
            "[8,64,8534533,{},\"wamp.error.invalid_argument\",[],{}]"
        );
    }

    #[test]
    fn serialize_subscribe() {
        two_way_test!(
            Message::Subscribe(
                58944,
                SubscribeOptions::new(),
                URI::new("ca.dal.test.the_sub")
            ),
            "[32,58944,{},\"ca.dal.test.the_sub\"]"
        )
    }

    #[test]
    fn serialize_subscribed() {
        two_way_test!(Message::Subscribed(47853, 48975938), "[33,47853,48975938]")
    }

    #[test]
    fn serialize_unsubscribe() {
        two_way_test!(Message::Unsubscribe(754, 8763), "[34,754,8763]")
    }

    #[test]
    fn serialize_unsubscribed() {
        two_way_test!(Message::Unsubscribed(675343), "[35,675343]")
    }

    #[test]
    fn serialize_publish() {
        two_way_test!(
            Message::Publish(
                453453,
                PublishOptions::new(false),
                URI::new("ca.dal.test.topic1"),
                None,
                None
            ),
            "[16,453453,{},\"ca.dal.test.topic1\"]"
        );

        two_way_test!(
            Message::Publish(
                23934583,
                PublishOptions::new(true),
                URI::new("ca.dal.test.topic2"),
                Some(vec![Value::String("a value".to_string())]),
                None
            ),
            "[16,23934583,{\"acknowledge\":true},\"ca.dal.test.topic2\",[\"a value\"]]"
        );
        let mut kwargs = HashMap::new();
        kwargs.insert("key1".to_string(), Value::List(vec![Value::Integer(-5)]));
        two_way_test!(
            Message::Publish(
                3243542,
                PublishOptions::new(true),
                URI::new("ca.dal.test.topic3"),
                Some(Vec::new()),
                Some(kwargs)
            ),
            "[16,3243542,{\"acknowledge\":true},\"ca.dal.test.topic3\",[],{\"key1\":[-5]}]"
        )
    }

    #[test]
    fn serialize_published() {
        two_way_test!(Message::Published(23443, 564564), "[17,23443,564564]")
    }

    #[test]
    fn serialize_event() {
        two_way_test!(
            Message::Event(4353453, 298173, EventDetails::new(), None, None),
            "[36,4353453,298173,{}]"
        );

        two_way_test!(
            Message::Event(
                764346,
                3895494,
                EventDetails::new(),
                Some(vec![Value::String("a value".to_string())]),
                None
            ),
            "[36,764346,3895494,{},[\"a value\"]]"
        );
        let mut kwargs = HashMap::new();
        kwargs.insert("key1".to_string(), Value::List(vec![Value::Integer(-5)]));
        two_way_test!(
            Message::Event(
                65675,
                587495,
                EventDetails::new(),
                Some(Vec::new()),
                Some(kwargs)
            ),
            "[36,65675,587495,{},[],{\"key1\":[-5]}]"
        )
    }

    #[test]
    fn serialize_register() {
        two_way_test!(
            Message::Register(25349185, RegisterOptions::new(), URI::new("ca.test.proc")),
            "[64,25349185,{},\"ca.test.proc\"]"
        );
    }

    #[test]
    fn serialize_registered() {
        two_way_test!(
            Message::Registered(25349185, 2103333224),
            "[65,25349185,2103333224]"
        );
    }

    #[test]
    fn serialize_unregister() {
        two_way_test!(
            Message::Unregister(788923562, 2103333224),
            "[66,788923562,2103333224]"
        );
    }

    #[test]
    fn serialize_unregistered() {
        two_way_test!(Message::Unregistered(788923562), "[67,788923562]");
    }

    #[test]
    fn serialize_call() {
        two_way_test!(
            Message::Call(
                7814135,
                CallOptions::new(),
                URI::new("com.myapp.ping"),
                None,
                None
            ),
            "[48,7814135,{},\"com.myapp.ping\"]"
        );

        two_way_test!(
            Message::Call(
                764346,
                CallOptions::new(),
                URI::new("com.myapp.echo"),
                Some(vec![Value::String("a value".to_string())]),
                None
            ),
            "[48,764346,{},\"com.myapp.echo\",[\"a value\"]]"
        );
        let mut kwargs = HashMap::new();
        kwargs.insert(
            "key1".to_string(),
            Value::List(vec![Value::UnsignedInteger(5)]),
        );
        two_way_test!(
            Message::Call(
                764346,
                CallOptions::new(),
                URI::new("com.myapp.compute"),
                Some(Vec::new()),
                Some(kwargs)
            ),
            "[48,764346,{},\"com.myapp.compute\",[],{\"key1\":[5]}]"
        )
    }

    #[test]
    fn serialize_invocation() {
        // two_way_test!(
        //     Message::Invocation(7814135, 9823526, InvocationDetails::new(), None, None),
        //     "[68,7814135,9823526,{}]"
        // );

        two_way_test!(
            Message::Invocation(
                764346,
                9823526,
                InvocationDetails::new(),
                Some(vec![Value::String("a value".to_string())]),
                None
            ),
            "[68,764346,9823526,{},[\"a value\"]]"
        );
        let mut kwargs = HashMap::new();
        kwargs.insert(
            "key1".to_string(),
            Value::List(vec![Value::UnsignedInteger(5)]),
        );
        two_way_test!(
            Message::Invocation(
                764346,
                9823526,
                InvocationDetails::new(),
                Some(Vec::new()),
                Some(kwargs)
            ),
            "[68,764346,9823526,{},[],{\"key1\":[5]}]"
        )
    }

    #[test]
    fn serialize_yield() {
        two_way_test!(
            Message::Yield(6131533, YieldOptions::new(), None, None),
            "[70,6131533,{}]"
        );

        two_way_test!(
            Message::Yield(
                6131533,
                YieldOptions::new(),
                Some(vec![Value::String("a value".to_string())]),
                None
            ),
            "[70,6131533,{},[\"a value\"]]"
        );
        let mut kwargs = HashMap::new();
        kwargs.insert(
            "key1".to_string(),
            Value::List(vec![Value::UnsignedInteger(5)]),
        );
        two_way_test!(
            Message::Yield(6131533, YieldOptions::new(), Some(Vec::new()), Some(kwargs)),
            "[70,6131533,{},[],{\"key1\":[5]}]"
        )
    }

    #[test]
    fn serialize_result() {
        two_way_test!(
            Message::Result(7814135, ResultDetails::new(), None, None),
            "[50,7814135,{}]"
        );

        two_way_test!(
            Message::Result(
                764346,
                ResultDetails::new(),
                Some(vec![Value::String("a value".to_string())]),
                None
            ),
            "[50,764346,{},[\"a value\"]]"
        );
        let mut kwargs = HashMap::new();
        kwargs.insert("key1".to_string(), Value::List(vec![Value::Float(8.6)]));
        two_way_test!(
            Message::Result(764346, ResultDetails::new(), Some(Vec::new()), Some(kwargs)),
            "[50,764346,{},[],{\"key1\":[8.6]}]"
        )
    }

}
