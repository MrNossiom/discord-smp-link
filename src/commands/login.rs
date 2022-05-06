//! Commands to link Discord and Google accounts together

use crate::{states::CommandResult, Context};
use poise::{
	command,
	serenity_prelude::{ButtonStyle, CollectComponentInteraction},
};
use std::time::Duration;

/// Connecte ton compte google SMP avec ton compte Discord pour vérifier ton identité.
#[command(slash_command)]
pub async fn login(ctx: Context<'_>) -> CommandResult {
	let (url, future) = ctx.data().auth.get_url_and_future();

	ctx.send(|reply| {
		reply
			.ephemeral(true)
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

	match future.await {
		Some(token) => {
			ctx.send(|reply| {
				reply.ephemeral(true).content(format!(
					"Google gave me this little secret : {}",
					token.secret()
				))
			})
			.await?;
		}
		None => {
			ctx.send(|reply| {
				reply
					.ephemeral(true)
					.content("You didn't finish the authentication process in 5 minutes.")
			})
			.await?;
		}
	};

	Ok(())
}

/// Dissocie ton compte Google SMP de ton compte Discord
#[command(slash_command)]
pub async fn logout(ctx: Context<'_>) -> CommandResult {
	/// The component id to retrieve the interaction
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
									.label("Disconnect your account")
									.custom_id(LOGOUT_COMPONENT_ID)
									.style(ButtonStyle::Danger)
							})
					})
				})
		})
		.await?;

	if let Some(interaction) = CollectComponentInteraction::new(ctx.discord())
		.message_id(reply.message().await?.id)
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
