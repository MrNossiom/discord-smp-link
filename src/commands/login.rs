//! Auth flow commands
//! links Discord and Google accounts together

use crate::{
	database::triggers,
	states::{InteractionResult, Shout},
	Context,
};
use anyhow::bail;
use poise::{
	command,
	serenity_prelude::{ButtonStyle, CollectComponentInteraction},
};
use std::time::Duration;

/// Connecte ton compte google SMP avec ton compte Discord pour vérifier ton identité
#[command(slash_command, guild_only, hide_in_help, member_cooldown = 10)]
pub async fn login(ctx: Context<'_>) -> InteractionResult {
	_login(ctx).await
}

/// Starts the auth process
///
/// Function used in the login and the setup command
pub async fn _login(ctx: Context<'_>) -> InteractionResult {
	let member = match ctx.author_member().await {
		Some(member) => member,
		None => bail!("You are not in a guild"),
	};

	let (oauth2_url, token_response) = ctx.data().auth.process_oauth2(Duration::from_secs(60 * 5));

	ctx.send(|reply| {
		reply
			.ephemeral(true)
			.content("Use your SMP account to connect yourself")
			.components(|components| {
				components.create_action_row(|action_row| {
					action_row.create_button(|button| {
						button
							.label("Continue")
							.url(oauth2_url)
							.style(ButtonStyle::Link)
					})
				})
			})
	})
	.await?;

	let token_response = match token_response.await {
		Some(response) => response,
		None => {
			ctx.shout("You didn't finish the authentication process in 5 minutes.".into())
				.await?;

			return Ok(());
		}
	};

	triggers::new_verified_member(&member, &token_response).await?;

	ctx.shout("You successfully authenticated with Google!".into())
		.await?;

	Ok(())
}

/// Dissocie ton compte Google SMP de ton compte Discord
#[command(slash_command, guild_only, hide_in_help, member_cooldown = 10)]
pub async fn logout(ctx: Context<'_>) -> InteractionResult {
	_logout(ctx).await
}

/// Starts the dissociate accounts process
///
/// Function used in the login and the setup command
pub async fn _logout(ctx: Context<'_>) -> InteractionResult {
	let reply = ctx
		.send(|reply| {
			reply
			.ephemeral(true)
			.content("After you disconnected your accounts, you will have to use the `/login` command again" )
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

	if let Some(interaction) = CollectComponentInteraction::new(ctx.discord())
		.message_id(reply.message().await?.id)
		// Use maximum duration from oauth response, 60 is tmp
		.timeout(Duration::from_secs(60))
		.await
	{
		interaction.defer(&ctx.discord().http).await?;

		if interaction.data.custom_id == "login.logout.disconnect" {
			triggers::delete_user(ctx.author())?;
		}
	}

	Ok(())
}
