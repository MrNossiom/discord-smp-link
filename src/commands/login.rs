use crate::{states::CommandResult, Context};
use poise::{
	command,
	serenity_prelude::{ButtonStyle, CollectComponentInteraction},
};
use std::time::Duration;

/// Connecte ton compte google SMP avec ton compte Discord pour vérifier ton identité.
#[command(slash_command)]
pub async fn login(ctx: Context<'_>) -> CommandResult {
	let reply = ctx
		.send(|reply| {
			reply
				.ephemeral(true)
				.content("Login into your SMP Google account\nAfter you authorized your account with the following link, enter the code with the /login slash command" )
				.components(|components| {
					components.create_action_row(|action_row| {
						action_row
							.create_button(|button| {
								button
									.label("Connect to Google")
									.url(ctx.data().auth.get_code())
									.style(ButtonStyle::Link)
							})
					})
				})
		})
		.await
		.unwrap();

	// TODO: Start oauth device flow

	CollectComponentInteraction::new(ctx.discord())
		.message_id(reply.message().await.unwrap().id)
		// Use maximum duration from oauth response, 60 is tmp
		.timeout(Duration::from_secs(60))
		.await;

	Ok(())
}
