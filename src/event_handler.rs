use futures::{stream, Future, Stream, StreamExt, TryStream, future::BoxFuture, channel::mpsc::{self, UnboundedSender}};
use std::collections::HashMap;
use ruma_client::{self, api, api::r0::sync::sync_events::IncomingResponse};
use ruma_client::events as ruma_events;
use ruma_events::collections::all::{Event, RoomEvent};
use ruma_events::room::message::MessageEventContent;
use ruma_client::identifiers::RoomId;
use log::{error, warn, debug};
use crate::error::{Error, InnerError};
use std::sync::Arc;


pub mod state;
pub mod message;

#[derive(Debug, Clone)]
pub struct Context {
    pub client: Arc<ruma_client::HttpsClient>,
}


pub trait EventHandler: Send + std::fmt::Debug  {
    fn handle(&self, ctx: &Context, input: &Event) -> Result<(), Error>;
}

pub trait StrippedHandler: Send + std::fmt::Debug  {
    fn handle(&self, ctx: &Context, room_id: RoomId, input: &ruma_events::stripped::StrippedState) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct RoomMessageEventHandler {
    pub handlers: HashMap<String, UnboundedSender<Result<ruma_events::room::message::MessageEvent, Error>>>,
    pub unknown_handler: Option<UnboundedSender<Result<ruma_events::room::message::MessageEvent, Error>>>,
}
impl RoomMessageEventHandler {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            unknown_handler: None
        }
    }

    /// Creates a new command and returns a stream which will yield a message when the command is send
    pub fn new_cmd(
        &mut self,
        cmd: &str,
    ) -> impl Stream<Item = Result<ruma_events::room::message::MessageEvent, Error>> {
        let (sender, receiver) = mpsc::unbounded();

        // let cmd = if cmd.starts_with("/") {
        //     cmd.into()
        // } else {
        //     format!("/{}", cmd)
        // };

        self.handlers.insert(cmd.into(), sender);

        receiver.map(|x| x.map_err(|_| Error(InnerError::Channel)))
    }

    /// Returns a stream which will yield a message when none of previously registered commands matches
    pub fn unknown_cmd(&mut self) -> impl Stream<Item = Result<ruma_events::room::message::MessageEvent, Error>> {
        let (sender, receiver) = mpsc::unbounded();

        self.unknown_handler = Some(sender);

        receiver.map(|x| x.map_err(|_| Error(InnerError::Channel)))
    }
}

impl EventHandler for RoomMessageEventHandler {
    fn handle(&self, ctx: &Context, input: &Event) -> Result<(), Error>{
        match input {
            Event::RoomMessage(input) => handle_input(ctx, input),
            _ => Err(Error(InnerError::Match))
        }
    }
}

fn handle_input(ctx: &Context, event: &ruma_events::room::message::MessageEvent) -> Result<(), Error> {
    use aho_corasick::{AhoCorasickBuilder, MatchKind};

    // client.receive_joined_timeline_event(&room_id, &e);


    // let event = Arc::new(event.clone());
    let mut sndr: Option<UnboundedSender<Result<ruma_events::room::message::MessageEvent, Error>>> = None;
    match event.content {
        MessageEventContent::Text(ref content) => {
            // let mut part = content.body.split_whitespace();
            // let patterns = self.handlers.keys().collect<Vec<_>>();
            // if let Some(mut cmd) = part.next() {
            //     if let Some(name) = self.name.as_ref() {
            //         if cmd.ends_with(name.as_str()) {
            //             cmd = cmd.rsplitn(2, '@').skip(1).next().unwrap();
            //         }
            //     }
            //     if let Some(sender) = self.handlers.get(cmd)
            //     {
            //         sndr = Some(sender.clone());
            //         message.text = Some(part.collect::<Vec<&str>>().join(" "));
            //     } else if let Some(ref sender) = self.unknown_handler
            //     {
            //         sndr = Some(sender.clone());
            //     }
            dbg!(content);
        }
        _ => {}
    }

    if let Some(sender) = sndr {
        sender
            .unbounded_send(Ok(event.clone()))
            .unwrap_or_else(|e| error!("Error: {}", e));
    }
    Ok(())
}




// pub struct ReactionEventHandler {
//     pub handlers: HashMap<String, UnboundedSender<api::r0::sync::sync_events::IncomingResponse, ruma_client::Error>>>>,
// }
// impl ReactionEventHandler {
//     pub fn new() -> Self{
//         Self {
//             handlers: HashMap::new(),
//         }
//     }
//     pub fn new_cmd(
//         &mut self,
//         cmd: &str,
//     ) -> impl Stream<Item = api::r0::sync::sync_events::IncomingResponse, Error = ruma_client::Error> {
//         let (sender, receiver) = mpsc::unbounded();

//         let cmd = if cmd.starts_with("/") {
//             cmd.into()
//         } else {
//             format!("/{}", cmd)
//         };

//         self.handlers.insert(cmd.into(), sender);

//         receiver.map_err(|_| crate::error::Error::Channel)
//     }
// }

// impl EventHandler for ReactionEventHandler {
//     type Streamable = ();

//     fn delegate()-> Self::Streamable {


//     }
// }