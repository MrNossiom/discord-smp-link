//! Setup messages for roles interactions

use crate::states::{Context, InteractionResult};
use diesel::update;
use poise::{command, serenity_prelude::component::ButtonStyle};

#[allow(clippy::missing_docs_in_private_items)]
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

	let reply_id = reply.message().await?.id.0;

	// Update the setup message id
	{
		use crate::database::schema::guilds::dsl::{guilds, setup_message_id};
		use diesel::prelude::*;

		update(guilds.find(ctx.interaction.guild_id().unwrap().0))
			.set(setup_message_id.eq(reply_id))
			.execute(&mut ctx.data.database.get()?)?;
	}

	// TODO: collect interactions in the event part

	Ok(())
}

/// Handles the class select action
async fn handle_class_select(ctx: Context<'_>) -> InteractionResult {
	ctx.send(|m| {
		m.embed(|cre| cre.title("Choose your class..."))
			.components(|com| {
				com.create_action_row(|row| {
					row.create_select_menu(|sel| {
						// TODO: add groups
						sel.options(|op| op.set_options(vec![]))
							.custom_id("setup.setup.class_selected")
					})
				})
			})
	})
	.await?;

	// TODO: handle button class selection
	todo!("handle button class select");
}
