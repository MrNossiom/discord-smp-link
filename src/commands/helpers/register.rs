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
#[command(
	slash_command,
	owners_only,
	hide_in_help,
	guild_only,
	rename = "register"
)]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(super) async fn debug_register(
	ctx: ApplicationContext<'_>,
	register: Option<bool>,
	global: Option<bool>,
) -> InteractionResult {
	let is_development_guild = ctx.guild_only_id() == ctx.data.config.discord_development_guild;

	let register = register.unwrap_or(true);
	let global = global.unwrap_or(false);

	let mut commands_builder = CreateApplicationCommands::default();

	for command in &ctx.framework.options.commands {
		if command.hide_in_help && !is_development_guild {
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
		Command::set_global_application_commands(&ctx.serenity_context, |b| {
			if register {
				*b = commands_builder;
			}
			b
		})
		.await?;
	} else {
		let Some(guild_id) = ctx.interaction.guild_id() else {
			ctx.shout(ctx.translate("error-guild-only", None)).await?;

			return Ok(());
		};

		guild_id
			.set_application_commands(&ctx.serenity_context, |b| {
				if register {
					*b = commands_builder;
				}
				b
			})
			.await?;
	}

	ctx.shout(ctx.translate("done", None)).await?;

	Ok(())
}
