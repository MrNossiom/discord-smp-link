//! Auth flow commands
//! links Discord and Google accounts together

use crate::{
	constants::AUTHENTICATION_TIMEOUT,
	database::triggers,
	handlers::auth::AuthProcessError,
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use anyhow::anyhow;
use poise::{
	command,
	serenity_prelude::{component::ButtonStyle, CollectComponentInteraction},
};
use std::{sync::Arc, time::Duration};

/// Starts the authentication flow
#[command(slash_command, guild_only)]
pub(crate) async fn login(ctx: ApplicationContext<'_>) -> InteractionResult {
	_login(ctx).await
}

/// Starts the auth process
///
/// Function used in the login and the setup command
pub(crate) async fn _login(ctx: ApplicationContext<'_>) -> InteractionResult {
	let member = match ctx.interaction.member() {
		Some(member) => member,
		None => {
			let msg = ctx.get("not-in-guild", None);
			ctx.shout(msg).await?;
			return Ok(());
		}
	};

	let (oauth2_url, token_response) = ctx.data.auth.process_oauth2(AUTHENTICATION_TIMEOUT);

	ctx.send(|reply| {
		reply
			.ephemeral(true)
			.content(ctx.get("use-google-account-to-login", None))
			.components(|components| {
				components.create_action_row(|action_row| {
					action_row.create_button(|button| {
						button
							.label("Continue")
							.style(ButtonStyle::Link)
							.url(oauth2_url)
					})
				})
			})
	})
	.await?;

	let token_response = match token_response.await {
		Ok(response) => response,
		Err(error) => match error {
			AuthProcessError::Timeout => {
				let msg = ctx.get("did-not-finish-auth-process", None);
				ctx.shout(msg).await?;

				return Ok(());
			}

			error => return Err(anyhow!("{error}")),
		},
	};

	triggers::new_verified_member(Arc::clone(ctx.data), member, &token_response).await?;

	let msg = ctx.get("authentication-successful", None);
	ctx.shout(msg).await?;

	Ok(())
}

/// Logs out the user
#[command(slash_command, guild_only)]
pub(crate) async fn logout(ctx: ApplicationContext<'_>) -> InteractionResult {
	_logout(ctx).await
}

/// Starts the dissociate accounts process
///
/// Function used in the login and the setup command
pub(crate) async fn _logout(ctx: ApplicationContext<'_>) -> InteractionResult {
	let reply = ctx
		.send(|reply| {
			reply
			.ephemeral(true)
			.content("After you disconnected your accounts, you will have to use the `/login` command again")
			.components(|components| {
				components.create_action_row(|action_row| {
					action_row
						.create_button(|button| {
							button
								.label("Disconnect your account")
								.custom_id("login.logout.disconnect")
								.style(ButtonStyle::Danger)
						})
				})
			})
		})
		.await?;

	if let Some(interaction) = CollectComponentInteraction::new(ctx.discord)
		.message_id(reply.message().await?.id)
		// Use maximum duration from oauth response, 60 is tmp
		.timeout(Duration::from_secs(60))
		.await
	{
		interaction.defer(&ctx.discord.http).await?;

		if interaction.data.custom_id == "login.logout.disconnect" {
			triggers::delete_user(Arc::clone(ctx.data), ctx.interaction.user())?;
		}
	}

	Ok(())
}
