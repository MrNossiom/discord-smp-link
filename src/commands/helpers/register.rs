//! Register or unregister all slash commands either globally or in a specific guild

use crate::states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult};
use poise::{
	command,
	serenity_prelude::{ButtonStyle, Command, CreateApplicationCommands},
};

/// Register or unregister all slash commands either globally or in a specific guild
#[command(slash_command, owners_only, hide_in_help)]
pub(super) async fn register(ctx: ApplicationContext<'_>) -> InteractionResult {
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
			ctx.shout("You didn't interact in time").await?;
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
				ctx.shout("Must be called in guild").await?;
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

	ctx.shout("Done!").await?;

	Ok(())
}
