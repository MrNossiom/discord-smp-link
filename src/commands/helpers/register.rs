//! Register or unregister all slash commands either globally or in a specific guild

use crate::{
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use poise::{
	command,
	serenity_prelude::{Command, CreateApplicationCommands},
};

/// Register or unregister all slash commands either globally or in a specific guild
#[command(slash_command, owners_only, hide_in_help)]
pub(super) async fn register(
	ctx: ApplicationContext<'_>,
	register: Option<bool>,
	global: Option<bool>,
) -> InteractionResult {
	let register = register.unwrap_or(true);
	let global = global.unwrap_or(false);

	let mut commands_builder = CreateApplicationCommands::default();

	for command in &ctx.framework.options.commands {
		if command.hide_in_help {
			continue;
		}

		if let Some(slash_command) = command.create_as_slash_command() {
			commands_builder.add_application_command(slash_command);
		}

		if let Some(context_menu_command) = command.create_as_context_menu_command() {
			commands_builder.add_application_command(context_menu_command);
		}
	}

	if global {
		if register {
			Command::set_global_application_commands(ctx.discord, |b| {
				*b = commands_builder;
				b
			})
			.await?;
		} else {
			Command::set_global_application_commands(ctx.discord, |b| b).await?;
		}
	} else {
		let guild_id = match ctx.interaction.guild_id() {
			Some(x) => x,
			None => {
				let get = ctx.get("error-guild-only", None);
				ctx.shout(get).await?;
				return Ok(());
			}
		};

		if register {
			guild_id
				.set_application_commands(ctx.discord, |b| {
					*b = commands_builder;
					b
				})
				.await?;
		} else {
			guild_id
				.set_application_commands(ctx.discord, |b| b)
				.await?;
		}
	}

	let get = ctx.get("done", None);
	ctx.shout(get).await?;

	Ok(())
}
