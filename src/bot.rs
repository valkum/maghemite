use std::sync::{Arc, RwLock};
use std::{env, process::exit};
use url::Url;
use std::collections::HashMap;
use log::{error, warn, debug};
use tokio;
use futures::{Stream, StreamExt, TryStreamExt, Future, FutureExt, TryFuture, TryFutureExt, channel::mpsc::{self, UnboundedSender}, stream, future::{self, join}};
use ruma_client::{self, *, api::r0::sync::sync_events::IncomingResponse};
use ruma_client::events as ruma_events;
use ruma_events::stripped::StrippedState;
use ruma_events::collections::all::{Event, RoomEvent};
use ruma_events::room::message::MessageEventContent;
use ruma_api::{Endpoint, Outgoing};
use std::rc::Rc;
use std::cell::RefCell;
use std::any::type_name;
use std::time::Duration;
use std::convert::TryFrom;
use http::Response as HttpResponse;

use crate::event_handler::{Context, EventHandler, RoomEventHandler, StateEventHandler, StrippedHandler};
use crate::error::Error;

pub struct BotConfig<'a> {
    homeserver_url: &'a str,
    session: Option<Session>,
    batch_id: Option<String>,
}

impl<'a> BotConfig<'a> {
    pub fn new(homeserver_url: &'a str, session: Option<Session>) -> Self{
        BotConfig { homeserver_url, session, batch_id: None }
    }
}

/// The main bot structure
///
/// Contains all configuration like `key`, `name`, etc. important handles to message the user and
/// `request` to issue requests to the Telegram server
#[derive(Debug)]
pub struct Bot<'a> {
    pub client: Arc<ruma_client::HttpsClient>,
    config: BotConfig<'a>,
    event_handlers: HashMap<ruma_client::events::EventType, Box<dyn EventHandler>>,
    stripped_handlers: HashMap<String, Box<dyn StrippedHandler>>,
}
impl<'a> Bot<'a> {
    pub fn new(config: BotConfig) -> Bot {
        // @todo user_agent(format!("RustyMatrixBot/{}", env!("CARGO_PKG_VERSION")))?
        // let hyper_client = hyper::Client::builder().keep_alive(true).build_http();
        let hyper_client = hyper::Client::builder().keep_alive(true).build(hyper_tls::HttpsConnector::new());
        let homeserver_url = Url::parse(&config.homeserver_url).unwrap();
        let client = Client::custom(hyper_client, homeserver_url, config.session, Some(Duration::from_secs(1)));


        Bot {
            client: Arc::new(client),
            config: config,
            event_handlers: HashMap::new(), 
            stripped_handlers: HashMap::new(),
        }
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<Session, ruma_client::Error> {
        // let session = self.client.session();
        let mut session = None;
        if session.is_none() {
            debug!("No session logging in as {}", username);
            session = Some(self.client.log_in(username.into(), password.into(), None, None).await?);
        }
        debug!("Logged in. Session is: {:?}", session);
        Ok(session.unwrap())
    }


    pub fn with_handler<H>(&mut self, event_type: ruma_client::events::EventType, handler: H) where H: EventHandler + 'static {
        debug!("Registered handler {:?}", type_name::<H>());
        self.event_handlers.insert(event_type, Box::new(handler));
    }

    pub fn with_stripped_handler<H>(&mut self, state_type: String, handler: H) where H: StrippedHandler + 'static {
        debug!("Registered handler {:?}", type_name::<H>());
        self.stripped_handlers.insert(state_type, Box::new(handler));
    }

    pub fn process_updates(self) -> impl Stream<Item = Result<Option<Event>, Error>> {
        self.client.sync(None, self.config.batch_id, true, Some(Duration::from_secs(10)))
        .map_ok(move |incoming| {
            let ctx = Context { client: self.client.clone() };
            
            for (room_id, room) in &incoming.rooms.invite {
                for event in &room.invite_state.events {
                    if let ruma_events::EventResult::Ok(e) = event {
                        // todo bring back type key for Hashmap
                        if let Some(handler) = self.stripped_handlers.get("test") {
                            handler.handle(&ctx, room_id.clone(), e);
                        } else {
                            debug!("Unhandled invite event. Register a RoomMember EventHandler")
                        } 
                    }
                }
            }
            for (room_id, room) in &incoming.rooms.join {
                // let room_id = room_id.to_string();
        
                let matrix_room = {
                    for event in &room.state.events {
                        if let ruma_events::EventResult::Ok(e) = event {
                            dbg!(e);
                            // self.client.receive_joined_state_event(&room_id, &e);
                        }
                    }
        
                    // self.client.joined_rooms.get(&room_id).unwrap().clone()
                };
        
                for event in &room.timeline.events {
                    if let ruma_events::EventResult::Ok(e) = event {
                        
                        if let Some(handler) = self.event_handlers.get(&ruma_events::EventType::RoomMessage) {
                            handler.handle(&ctx, None, e);
                        } else {
                            debug!("Unhandled RoomMessage event. Register a RoomMessage EventHandler")
                        } 
                        // if let Err(err) = handlers::handle(&ctx, &event).await {
                        //     match err {
                        //         HandlerError::Message(message) => {
                        //             if let Some(issue) = event.issue() {
                        //                 let cmnt = ErrorComment::new(issue, message);
                        //                 cmnt.post(&ctx.github).await?;
                        //             }
                        //         }
                        //         HandlerError::Other(err) => {
                        //             log::error!("handling event failed: {:?}", err);
                        //             return Err(WebhookError(anyhow::anyhow!(
                        //                 "handling failed, error logged",
                        //             )));
                        //         }
                        //     }
                        // }
                    }
                }
            }
            for (room_id, room) in &incoming.rooms.leave {
                for event in &room.state.events {
                    if let ruma_events::EventResult::Ok(e) = event {
                        dbg!(e);
                    }
                }
                for event in &room.timeline.events {
                    if let ruma_events::EventResult::Ok(e) = event {
                        dbg!(e);
                    }
                }
            }
            stream::once(async { Ok(None) })
            // async { Ok(()) }

        }).try_flatten()
        // todo: 
        // stream::once(async { Ok(None) })
    }

    pub fn into_future(self) -> impl Future<Output = ()> {
        self.process_updates().for_each(|_| async {()}).map(|_| ())
    }
}