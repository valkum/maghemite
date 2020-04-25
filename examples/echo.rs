use futures::StreamExt;
use futures::stream::Stream;
use futures::prelude::*;
use crate::bot::*;
use app_dirs::*;
use tokio::prelude::*;

const APP_INFO: AppInfo = AppInfo{name: "echo", author: "maghemite"};

#[tokio::main]
async fn main() {
    let config = BotConfig {
        homeserver_url: "https://matrix.org/",
        session: None
    };

    let mut bot = Bot::new(config);
    // @todo save session
    let mut message_handler = event_handler::RoomMessageEventHandler::new();
    

    let new_handle = message_handler.new_cmd("!cal new event")
        .and_then(|(msg)| {
            // let mut text = msg.text.unwrap().clone();
            // if text.is_empty() {
            //     text = "<empty>".into();
            // }

            // bot.message(msg.chat.id, text).send()
            dbg!(msg)
        })
        .for_each(|_| Ok(()));

    bot.with_handler(ruma_client::events::EventType::RoomMessage, &message_handler);
    bot.run_with(new_handle);
}