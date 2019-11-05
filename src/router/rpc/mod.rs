use std::sync::Arc;

use log::{debug, info};

use crate::messages::{
    CallOptions, ErrorType, InvocationDetails, Message, Reason, RegisterOptions, ResultDetails,
    YieldOptions, URI,
};
use crate::{Dict, Error, ErrorKind, List, MatchingPolicy, WampResult, ID};

use super::messaging::send_message;
use super::{random_id, ConnectionHandler};

mod patterns;
pub use self::patterns::RegistrationPatternNode;

impl ConnectionHandler {
    pub fn handle_register(
        &mut self,
        request_id: ID,
        options: RegisterOptions,
        procedure: URI,
    ) -> WampResult<()> {
        debug!(
            "Responding to register message (id: {}, procedure: {})",
            request_id, procedure.uri
        );
        match self.realm {
            Some(ref realm) => {
                let mut realm = realm.lock().unwrap();
                let manager = &mut realm.registration_manager;
                let procedure_id = {
                    let procedure_id = match manager.registrations.register_with(
                        &procedure,
                        Arc::clone(&self.info),
                        options.pattern_match,
                        options.invocation_policy,
                    ) {
                        Ok(procedure_id) => procedure_id,
                        Err(e) => {
                            return Err(Error::new(ErrorKind::ErrorReason(
                                ErrorType::Register,
                                request_id,
                                e.reason(),
                            )))
                        }
                    };
                    self.registered_procedures.push(procedure_id);
                    procedure_id
                };
                manager.registration_ids_to_uris.insert(
                    procedure_id,
                    (
                        procedure.uri,
                        options.pattern_match == MatchingPolicy::Prefix,
                    ),
                );
                send_message(&self.info, &Message::Registered(request_id, procedure_id))
            }
            None => Err(Error::new(ErrorKind::InvalidState(
                "Received a message while not attached to a realm",
            ))),
        }
    }

    pub fn handle_unregister(&mut self, request_id: ID, procedure_id: ID) -> WampResult<()> {
        match self.realm {
            Some(ref realm) => {
                let mut realm = realm.lock().unwrap();
                let manager = &mut realm.registration_manager;
                let (procedure_uri, is_prefix) =
                    match manager.registration_ids_to_uris.get(&procedure_id) {
                        Some(&(ref uri, is_prefix)) => (uri.clone(), is_prefix),
                        None => {
                            return Err(Error::new(ErrorKind::ErrorReason(
                                ErrorType::Unregister,
                                request_id,
                                Reason::NoSuchProcedure,
                            )))
                        }
                    };

                let procedure_id = match manager.registrations.unregister_with(
                    &procedure_uri,
                    &self.info,
                    is_prefix,
                ) {
                    Ok(procedure_id) => procedure_id,
                    Err(e) => {
                        return Err(Error::new(ErrorKind::ErrorReason(
                            ErrorType::Unregister,
                            request_id,
                            e.reason(),
                        )))
                    }
                };
                self.registered_procedures.retain(|id| *id != procedure_id);
                send_message(&self.info, &Message::Unregistered(request_id))
            }
            None => Err(Error::new(ErrorKind::InvalidState(
                "Received a message while not attached to a realm",
            ))),
        }
    }

    pub fn handle_call(
        &mut self,
        request_id: ID,
        _options: CallOptions,
        procedure: URI,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) -> WampResult<()> {
        debug!(
            "Responding to call message (id: {}, procedure: {})",
            request_id, procedure.uri
        );
        match self.realm {
            Some(ref realm) => {
                let mut realm = realm.lock().unwrap();
                let manager = &mut realm.registration_manager;
                let invocation_id = random_id();
                info!("Current procedure tree: {:?}", manager.registrations);
                let (registrant, procedure_id, policy) =
                    match manager.registrations.get_registrant_for(procedure.clone()) {
                        Ok(registrant) => registrant,
                        Err(e) => {
                            return Err(Error::new(ErrorKind::ErrorReason(
                                ErrorType::Call,
                                request_id,
                                e.reason(),
                            )))
                        }
                    };
                manager
                    .active_calls
                    .insert(invocation_id, (request_id, Arc::clone(&self.info)));
                let mut details = InvocationDetails::new();
                details.procedure = if policy == MatchingPolicy::Strict {
                    None
                } else {
                    Some(procedure)
                };
                let invocation_message =
                    Message::Invocation(invocation_id, procedure_id, details, args, kwargs);
                send_message(registrant, &invocation_message)?;

                Ok(())
            }
            None => Err(Error::new(ErrorKind::InvalidState(
                "Received a message while not attached to a realm",
            ))),
        }
    }

    pub fn handle_yield(
        &mut self,
        invocation_id: ID,
        _options: YieldOptions,
        args: Option<List>,
        kwargs: Option<Dict>,
    ) -> WampResult<()> {
        debug!("Responding to yield message (id: {})", invocation_id);
        match self.realm {
            Some(ref realm) => {
                let mut realm = realm.lock().unwrap();
                let manager = &mut realm.registration_manager;
                if let Some((call_id, callee)) = manager.active_calls.remove(&invocation_id) {
                    let result_message =
                        Message::Result(call_id, ResultDetails::new(), args, kwargs);
                    send_message(&callee, &result_message)
                } else {
                    Err(Error::new(ErrorKind::InvalidState(
                        "Received a yield message for a call that wasn't sent",
                    )))
                }
            }
            None => Err(Error::new(ErrorKind::InvalidState(
                "Received a message while not attached to a realm",
            ))),
        }
    }
}
