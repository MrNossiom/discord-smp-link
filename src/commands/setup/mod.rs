//! A set of commands to manage the bot.

use crate::states::{ApplicationContext, InteractionResult};
use poise::command;

mod message;
mod role;

use message::setup_message;
use role::setup_role;

/// A set of commands to setup the bot
#[allow(clippy::unused_async)]
#[command(
	slash_command,
	rename = "setup",
	subcommands("setup_message", "setup_role"),
	default_member_permissions = "ADMINISTRATOR"
)]
pub(crate) async fn setup(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}
