//! Act on discord client metadata

use crate::states::{ApplicationContext, InteractionResult, PrefixContext};
use poise::command;

mod force;
mod refresh;
mod register;

use force::force;
use refresh::refresh;
use register::register;

/// A set of commands restricted to owners
/// Can be registered with [`_register`] prefix command
#[allow(clippy::unused_async)]
#[command(
	slash_command,
	owners_only,
	hide_in_help,
	subcommands("force", "refresh", "register")
)]
pub(crate) async fn debug(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Register all development slash commands
#[command(prefix_command, guild_only, owners_only)]
pub(crate) async fn _register(ctx: PrefixContext<'_>) -> InteractionResult {
	let guild_id = ctx.msg.guild_id.expect("this command is guild only");

	for command in &ctx.framework.options.commands {
		if !command.hide_in_help {
			continue;
		}

		if let Some(slash_command) = command.create_as_slash_command() {
			guild_id
				.create_application_command(ctx.discord, |c| {
					*c = slash_command;
					c
				})
				.await?;
		}

		if let Some(context_menu_command) = command.create_as_context_menu_command() {
			guild_id
				.create_application_command(ctx.discord, |c| {
					*c = context_menu_command;
					c
				})
				.await?;
		}
	}

	ctx.msg.delete(ctx.discord).await?;

	Ok(())
}
