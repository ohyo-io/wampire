use std::sync::Arc;

use log::{debug, info, warn};
use ws::{
    CloseCode, Error as WSError, ErrorKind as WSErrorKind, Request, Response, Result as WSResult,
};

use crate::messages::{
    ErrorDetails, HelloDetails, Message, Reason, RouterRoles, WelcomeDetails, URI,
};
use crate::router::messaging::send_message;
use crate::{Error, ErrorKind, WampResult};

use super::{ConnectionHandler, ConnectionState, WAMP_JSON, WAMP_MSGPACK};

impl ConnectionHandler {
    pub fn handle_hello(&mut self, realm: URI, _details: HelloDetails) -> WampResult<()> {
        debug!("Responding to hello message (realm: {:?})", realm);
        let id = {
            let mut info = self.info.lock().unwrap();
            info.state = ConnectionState::Connected;
            info.id
        };

        self.set_realm(realm.uri)?;
        send_message(
            &self.info,
            &Message::Welcome(id, WelcomeDetails::new(RouterRoles::new())),
        )
    }

    pub fn handle_goodbye(&mut self, _details: ErrorDetails, reason: Reason) -> WampResult<()> {
        let state = self.info.lock().unwrap().state.clone();
        match state {
            ConnectionState::Initializing => {
                // TODO check specification for how this ought to work.
                Err(Error::new(ErrorKind::InvalidState(
                    "Received a goodbye message before handshake complete",
                )))
            }
            ConnectionState::Connected => {
                info!("Received goodbye message with reason: {:?}", reason);
                self.remove();
                send_message(
                    &self.info,
                    &Message::Goodbye(ErrorDetails::new(), Reason::GoodbyeAndOut),
                )
                .ok();
                let mut info = self.info.lock().unwrap();
                info.state = ConnectionState::Disconnected;
                match info.sender.close(CloseCode::Normal) {
                    Err(e) => Err(Error::new(ErrorKind::WSError(e))),
                    _ => Ok(()),
                }
            }
            ConnectionState::ShuttingDown => {
                info!(
                    "Received goodbye message in response to our goodbye message with reason: {:?}",
                    reason
                );
                let mut info = self.info.lock().unwrap();
                info.state = ConnectionState::Disconnected;
                match info.sender.close(CloseCode::Normal) {
                    Err(e) => Err(Error::new(ErrorKind::WSError(e))),
                    _ => Ok(()),
                }
            }
            ConnectionState::Disconnected => {
                warn!("Received goodbye message after closing connection");
                Ok(())
            }
        }
    }

    fn set_realm(&mut self, realm: String) -> WampResult<()> {
        debug!("Setting realm to {}", realm);
        if let Some(realm) = self.router.realms.lock().unwrap().get(&realm) {
            {
                realm
                    .lock()
                    .unwrap()
                    .connections
                    .push(Arc::clone(&self.info));
            }
            self.realm = Some(Arc::clone(realm));
        } else {
            return Err(Error::new(ErrorKind::HandshakeError(Reason::NoSuchRealm)));
        }
        Ok(())
    }

    pub fn process_protocol(&mut self, request: &Request, response: &mut Response) -> WSResult<()> {
        debug!("Checking protocol");
        let protocols = request.protocols()?;
        for protocol in protocols {
            if protocol == WAMP_JSON || protocol == WAMP_MSGPACK {
                response.set_protocol(protocol);
                let mut info = self.info.lock().unwrap();
                info.protocol = protocol.to_string();
                return Ok(());
            }
        }
        Err(WSError::new(
            WSErrorKind::Protocol,
            format!(
                "Neither {} nor {} were selected as Websocket sub-protocols",
                WAMP_JSON, WAMP_MSGPACK
            ),
        ))
    }
}
