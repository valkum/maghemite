pub use event_handler::{EventHandler};
pub use bot::{Bot, BotConfig};
pub use error::Error;

pub mod bot;
pub mod event_handler;
pub mod error;
mod compat;