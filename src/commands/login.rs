use crate::{states::CommandResult, Context};
use poise::{
	command,
	serenity_prelude::{ButtonStyle, CollectComponentInteraction},
};
use std::time::Duration;

/// Connecte ton compte google SMP avec ton compte Discord pour vérifier ton identité.
#[command(slash_command)]
pub async fn login(ctx: Context<'_>) -> CommandResult {
	const CONTINUE_BUTTON_ID: &str = "continue";
	const CANCEL_BUTTON_ID: &str = "cancel";

	let reply = ctx
		.send(|reply| {
			reply
				.ephemeral(true)
				.content("Login into your SMP Google account by clicking the button below.")
				.components(|components| {
					components.create_action_row(|action_row| {
						action_row
							.create_button(|button| {
								button
									.label("Continue")
									.custom_id(CONTINUE_BUTTON_ID)
									.style(ButtonStyle::Success)
							})
							.create_button(|button| {
								button
									.label("Cancel")
									.custom_id(CANCEL_BUTTON_ID)
									.style(ButtonStyle::Secondary)
							})
					})
				})
		})
		.await?;

	let (url, future) = ctx.data().auth.get_url_and_future();

	if let Some(interaction) = CollectComponentInteraction::new(ctx.discord())
		.message_id(reply.message().await.unwrap().id)
		.timeout(Duration::from_secs(60))
		.await
	{
		interaction.defer(&ctx.discord().http).await?;

		match interaction.data.custom_id.as_str() {
			CONTINUE_BUTTON_ID => {
				ctx.send(|msg| {
					msg.ephemeral(true)
						.content("Use your SMP account to connect yourself")
						.components(|components| {
							components.create_action_row(|action_row| {
								action_row.create_button(|button| {
									button.label("Continue").url(url).style(ButtonStyle::Link)
								})
							})
						})
				})
				.await?;
			}
			CANCEL_BUTTON_ID => {
				ctx.send(|msg| msg.ephemeral(true).content("Ok")).await?;

				return Ok(());
			}
			&_ => unreachable!(),
		}
	}

	dbg!(&future.await);

	Ok(())
}

/// Dissocie ton compte Google SMP de ton compte Discord
#[command(slash_command)]
pub async fn logout(ctx: Context<'_>) -> CommandResult {
	const LOGOUT_COMPONENT_ID: &str = "logout";

	let reply = ctx
		.send(|reply| {
			reply
				.ephemeral(true)
				.content("After you disconnected your accounts, you will have to use the /login command again" )
				.components(|components| {
					components.create_action_row(|action_row| {
						action_row
							.create_button(|button| {
								button
									.label("Disconnect your accounts")
									.custom_id(LOGOUT_COMPONENT_ID)
									.style(ButtonStyle::Danger)
							})
					})
				})
		})
		.await?;

	if let Some(interaction) = CollectComponentInteraction::new(ctx.discord())
		.message_id(reply.message().await.unwrap().id)
		// Use maximum duration from oauth response, 60 is tmp
		.timeout(Duration::from_secs(60))
		.await
	{
		interaction.defer(&ctx.discord().http).await?;

		if interaction.data.custom_id == LOGOUT_COMPONENT_ID {
			ctx.send(|msg| {
				msg.ephemeral(true)
					.content("You will be logout once this is implemented")
			})
			.await?;

			todo!("logout command");
		}
	}

	Ok(())
}
