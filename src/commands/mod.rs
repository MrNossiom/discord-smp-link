//! `Discord` client commands

use crate::{
	states::{Context, ContextPolyfill, FrameworkError},
	translation::Translate,
};
use anyhow::{anyhow, Context as _};
use console::style;
use fluent::fluent_args;
use poise::BoxFuture;
use uuid::Uuid;

mod information;
mod setup;

pub(crate) use information::information;
pub(crate) use setup::setup;
pub(crate) mod helpers;
pub(crate) mod login;

/// Execute before each command
pub(crate) fn pre_command(ctx: Context) -> BoxFuture<()> {
	Box::pin(async move {
		tracing::info!(
			"{} invoked {}",
			style(&ctx.author().name).black().bright(),
			style(ctx.invoked_command_name()).black().bright(),
		);
	})
}

/// Execute on a error during code execution
pub(crate) fn command_on_error(error: FrameworkError) -> BoxFuture<()> {
	Box::pin(async move {
		let error = match error {
			FrameworkError::Command { error, ctx, .. } => {
				let error_identifier = Uuid::new_v4().hyphenated().to_string();

				tracing::error!(
					"[id: {}] {} invoked `{}` but an error occurred: {}",
					error_identifier,
					ctx.author().name,
					ctx.invoked_command_name(),
					error
				);

				let error_msg = ctx.get(
					"internal-error-with-id",
					Some(&fluent_args!["id" => error_identifier]),
				);

				ctx.shout(error_msg)
					.await
					.map(|_| ())
					.context("Failed to send internal error message")
			}

			FrameworkError::ArgumentParse { error, .. } => {
				tracing::error!("Argument Parse: {}", error);

				Ok(())
			}

			FrameworkError::CommandStructureMismatch { description, .. } => {
				tracing::error!(
					"Command Structure Mismatch: You should sync your commands: {} ",
					description
				);

				Ok(())
			}

			FrameworkError::CooldownHit {
				remaining_cooldown,
				ctx,
			} => ctx
				.shout(format!(
					"You can use this command again in {} seconds",
					remaining_cooldown.as_secs()
				))
				.await
				.map(|_| ())
				.context("Failed to send cooldown hit message"),

			FrameworkError::MissingBotPermissions {
				ctx,
				missing_permissions,
			} => ctx
				.shout(format!(
					"The bot is missing the following permissions: {}",
					missing_permissions
				))
				.await
				.map(|_| ())
				.context("Failed to send missing bot permissions message"),

			FrameworkError::MissingUserPermissions {
				ctx,
				missing_permissions,
			} => ctx
				.shout(format!(
					"You are missing the following permissions : {}",
					missing_permissions.unwrap_or_default()
				))
				.await
				.map(|_| ())
				.context("Failed to send missing user permissions message"),

			FrameworkError::NotAnOwner { ctx } => ctx
				.shout("You are not the owner of this bot.".into())
				.await
				.map(|_| ())
				.context("Failed to send not an owner message"),

			FrameworkError::GuildOnly { ctx } => ctx
				.shout("This command can only be used in a guild channel".into())
				.await
				.map(|_| ())
				.context("Failed to send guild only message"),

			FrameworkError::DmOnly { ctx } => ctx
				.shout("This command can only be used in a DM channel".into())
				.await
				.map(|_| ())
				.context("Failed to send dm only message"),

			FrameworkError::CommandCheckFailed { ctx, error } => {
				let error_identifier = Uuid::new_v4().hyphenated().to_string();

				tracing::error!(
					"[id: {}] {} invoked `{}` but an error occurred in command check : {}",
					error_identifier,
					ctx.author().name,
					ctx.invoked_command_name(),
					error.unwrap_or_else(|| anyhow!("Unknown error"))
				);

				let error_msg = ctx.get(
					"internal-error-with-id",
					Some(&fluent_args!["id" => error_identifier]),
				);

				ctx.shout(error_msg)
					.await
					.map(|_| ())
					.context("Failed to send command check failed message")
			}

			_ => Ok(()),
		};

		if let Err(error) = error {
			tracing::error!("{}", error);
		};
	})
}

/// Execute after every successful command
pub(crate) fn post_command(ctx: Context) -> BoxFuture<()> {
	Box::pin(async move {
		tracing::debug!(
			"{} invoked `{}` successfully!",
			style(&ctx.author().name).black().bright(),
			style(ctx.invoked_command_name()).black().bright(),
		);
	})
}
