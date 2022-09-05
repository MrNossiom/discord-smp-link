//! Auth flow event handlers
//! Links Discord and Google accounts together

use crate::{
	constants::AUTHENTICATION_TIMEOUT,
	database::{schema::members, triggers},
	handlers::auth::AuthProcessError,
	states::{InteractionResult, MessageComponentContext},
	translation::Translate,
};
use anyhow::anyhow;
use diesel::prelude::*;
use poise::serenity_prelude::{component::ButtonStyle, CollectComponentInteraction};
use std::{sync::Arc, time::Duration};

/// Starts the auth process
///
/// Function used in the login and the setup command
pub(crate) async fn login(ctx: MessageComponentContext<'_>) -> InteractionResult {
	let member = match &ctx.interaction.member {
		Some(member) => member,
		None => {
			ctx.send(|c| c.content(ctx.get("error-guild-only", None)))
				.await?;

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
				let content = ctx.get("did-not-finish-auth-process", None);
				ctx.shout(content).await?;

				return Ok(());
			}

			error => return Err(anyhow!("{error}")),
		},
	};

	triggers::new_verified_member(Arc::clone(ctx.data), member, &token_response).await?;

	let content = ctx.get("authentication-successful", None);
	ctx.shout(content).await?;

	Ok(())
}

/// Starts the dissociate accounts process
///
/// Function used in the login and the setup command
pub(crate) async fn logout(ctx: MessageComponentContext<'_>) -> InteractionResult {
	let reply = ctx
		.send(|reply| {
			reply
				.ephemeral(true)
				.content(ctx.get("logout-warning", None))
				.components(|components| {
					components.create_action_row(|action_row| {
						action_row.create_button(|button| {
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
		match &*interaction.data.custom_id {
			// TODO: change cast to be functional
			"login.logout.disconnect" => diesel::delete(members::table)
				.filter(members::discord_id.eq(ctx.interaction.user.id.0))
				.execute(&mut ctx.data.database.get()?)?,

			_ => unreachable!(),
		};
	}

	Ok(())
}
