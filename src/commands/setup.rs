//! Setup messages for roles interactions

use crate::{
	commands::login::{_login, _logout},
	states::{Context, InteractionResult},
};
use anyhow::anyhow;
use futures::StreamExt;
use lazy_static::lazy_static;
use poise::{
	command,
	serenity_prelude::{
		component::ButtonStyle, ComponentInteractionCollectorBuilder, CreateSelectMenuOption,
	},
};

lazy_static! {
	static ref CLASSES_OP: Vec<CreateSelectMenuOption> =
		["101", "102", "103", "104", "105", "106", "107", "108", "109", "110", "111"]
			.iter()
			.map(|id| CreateSelectMenuOption::new(id, id))
			.collect();
}

/// Sets up the interaction roles & login & logout message
#[command(
	slash_command,
	guild_only,
	hide_in_help,
	required_permissions = "ADMINISTRATOR"
)]
pub async fn setup(ctx: Context<'_>) -> InteractionResult {
	let reply = ctx
		.send(|m| {
			m.embed(|cre| cre.title("Choose your level..."))
				.components(|com| {
					com.create_action_row(|row| {
						row.create_button(|butt| {
							butt.label("Login")
								.style(ButtonStyle::Success)
								.custom_id("setup.setup.login")
						})
						.create_button(|butt| {
							butt.label("Logout")
								.style(ButtonStyle::Danger)
								.custom_id("setup.setup.logout")
						})
						.create_button(|butt| {
							butt.label("Choose class")
								.style(ButtonStyle::Primary)
								.custom_id("setup.setup.class_select")
						})
					})
				})
		})
		.await?;

	let mut interactions = ComponentInteractionCollectorBuilder::new(ctx.discord())
		.message_id(reply.message().await?.id)
		.build();

	while let Some(interaction) = interactions.next().await {
		interaction.defer(&ctx.discord().http).await?;

		// Handle button click
		match interaction.data.custom_id.as_str() {
			"setup.setup.login" => _login(ctx).await,
			"setup.setup.logout" => _logout(ctx).await,
			"setup.setup.choose_class" => handle_class_select(ctx).await,

			msg => Err(anyhow!("Unknown custom id : {}", msg)),
		}?;
	}

	Ok(())
}

/// Handles the class select action
async fn handle_class_select(ctx: Context<'_>) -> InteractionResult {
	ctx.send(|m| {
		m.embed(|cre| cre.title("Choose your class..."))
			.components(|com| {
				com.create_action_row(|row| {
					row.create_select_menu(|sel| {
						sel.options(|op| op.set_options((*CLASSES_OP).clone()))
							.custom_id("setup.setup.class_selected")
					})
				})
			})
	})
	.await?;

	// TODO: handle button class selection
	todo!("handle button class select");
}
