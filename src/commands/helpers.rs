//! Act on discord client metadata

use crate::states::{Context, InteractionResult};
use poise::{
	builtins::{self, register_application_commands, HelpConfiguration},
	command,
	serenity_prelude::ApplicationCommand,
};

/// Register all slash commands to `Discord` either globally or in a specific guild
#[command(prefix_command, owners_only)]
pub async fn register(ctx: Context<'_>, #[flag] global: bool) -> InteractionResult {
	register_application_commands(ctx, global).await?;

	Ok(())
}

/// Unregister all global slash commands from `Discord`
#[command(prefix_command, owners_only)]
pub async fn reset_global(ctx: Context<'_>) -> InteractionResult {
	ApplicationCommand::set_global_application_commands(ctx.discord(), |b| b).await?;

	Ok(())
}

/// Show help about internal commands
#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn help(
	ctx: Context<'_>,
	#[description = "Specific command to show help about"] command: Option<String>,
) -> InteractionResult {
	let config = HelpConfiguration {
		show_context_menu_commands: true,
		..Default::default()
	};

	builtins::help(ctx, command.as_deref(), config).await?;

	Ok(())
}
