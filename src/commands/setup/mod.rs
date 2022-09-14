//! A set of commands to manage the bot.

use crate::states::{ApplicationContext, InteractionResult};
use poise::command;

mod message;
mod role;

use message::message;
use role::role;

/// A set of commands to setup the bot
#[allow(clippy::unused_async)]
#[command(slash_command, subcommands("message", "role"))]
pub(crate) async fn setup(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}
