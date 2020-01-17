use futures::{stream, Future, Stream, StreamExt, TryStream, future::BoxFuture, channel::mpsc::{self, UnboundedSender}};
use std::collections::HashMap;
use ruma_client::{self, api, api::r0::sync::sync_events::IncomingResponse};
use ruma_client::events as ruma_events;
use ruma_events::collections::all::{Event, RoomEvent};
use ruma_events::stripped::{StrippedState, StrippedRoomMember};
use ruma_events::room::member::MemberEvent;
use ruma_events::room::member::MemberEventContent;
use ruma_client::identifiers::RoomId;
use log::{error, warn, debug};
use crate::error::{Error, InnerError};

use crate::event_handler::{StrippedHandler, Context};

#[derive(Debug)]
pub struct RoomMemberEventHandler {
    pub handlers: HashMap<ruma_events::EventType, UnboundedSender<Result<(Context, RoomId, StrippedRoomMember), Error>>>
}

impl RoomMemberEventHandler {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    pub fn add(&mut self, cmd: ruma_events::EventType) -> impl Stream<Item = Result<(Context, RoomId, StrippedRoomMember), Error>> {
        let (sender, receiver) = mpsc::unbounded();

        // let cmd = if cmd.starts_with("/") {
        //     cmd.into()
        // } else {
        //     format!("/{}", cmd)
        // };

        self.handlers.insert(cmd.into(), sender);

        receiver.map(|x| x.map_err(|_| Error(InnerError::Channel)))
    }
    fn handle_input(&self, ctx: &Context, room_id: RoomId, event: &StrippedRoomMember) -> Result<(), Error> {
        // let event = Arc::new(event.clone());
        dbg!(event);
        if let Some(sender) = self.handlers.get(&event.event_type)
        {
            sender
                .unbounded_send(Ok((ctx.clone(), room_id, event.clone())))
                .unwrap_or_else(|e| error!("Error: {}", e));
        }
        Ok(())
    }
}

impl StrippedHandler for RoomMemberEventHandler {
    fn handle(&self, ctx: &Context, room_id: RoomId, input: &StrippedState) -> Result<(), Error>{
        match input {
            StrippedState::RoomMember(input) => self.handle_input(ctx, room_id, input),
            _ => Err(Error(InnerError::Match))
        }
    }
}