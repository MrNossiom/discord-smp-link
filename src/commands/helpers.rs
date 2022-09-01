//! Act on discord client metadata

use crate::states::{
	ApplicationContext, ApplicationContextPolyfill, InteractionResult, PrefixContext,
};
use poise::{
	command,
	serenity_prelude::{ButtonStyle, Command, CreateApplicationCommands},
};

/// Register all development slash commands
#[command(prefix_command, owners_only, guild_only)]
pub(crate) async fn register_dev(ctx: PrefixContext<'_>) -> InteractionResult {
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

	Ok(())
}

/// A set of commands restricted to owners
/// Can be registered with [`register_dev`] prefix command
#[allow(clippy::unused_async)]
#[command(slash_command, owners_only, hide_in_help, subcommands("register"))]
pub(crate) async fn dev(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Register all slash commands to `Discord` either globally or in a specific guild
#[command(slash_command, owners_only, hide_in_help)]
async fn register(ctx: ApplicationContext<'_>) -> InteractionResult {
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

	let reply = ctx
		.send(|m| {
			m.ephemeral(true)
				.content("Choose what to do with the commands:")
				.components(|c| {
					c.create_action_row(|r| {
						r.create_button(|b| {
							b.custom_id("register.global")
								.label("Register globally")
								.style(ButtonStyle::Primary)
						})
						.create_button(|b| {
							b.custom_id("unregister.global")
								.label("Delete globally")
								.style(ButtonStyle::Danger)
						})
						.create_button(|b| {
							b.custom_id("register.guild")
								.label("Register in guild")
								.style(ButtonStyle::Primary)
						})
						.create_button(|b| {
							b.custom_id("unregister.guild")
								.label("Delete in guild")
								.style(ButtonStyle::Danger)
						})
					})
				})
		})
		.await?;

	let interaction = reply
		.message()
		.await?
		.await_component_interaction(ctx.discord)
		.await;

	let pressed_button_id = match &interaction {
		Some(interaction) => {
			interaction.defer(ctx.discord).await?;
			&interaction.data.custom_id
		}
		None => {
			ctx.shout("You didn't interact in time".into()).await?;
			return Ok(());
		}
	};

	let (register, global) = match &**pressed_button_id {
		"register.global" => (true, true),
		"unregister.global" => (false, true),
		"register.guild" => (true, false),
		"unregister.guild" => (false, false),
		_ => unreachable!(),
	};

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
				ctx.shout("Must be called in guild".into()).await?;
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

	ctx.shout("Done!".into()).await?;

	Ok(())
}
