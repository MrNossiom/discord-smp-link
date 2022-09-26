//! `Discord` client commands

use crate::{
	states::{Context, ContextPolyfill, FrameworkError},
	translation::Translate,
};
use anyhow::{anyhow, Context as _};
use fluent::fluent_args;
use poise::BoxFuture;
use uuid::Uuid;

mod information;
mod setup;

pub(crate) use information::information;
pub(crate) use setup::setup;
pub(crate) mod helpers;

/// Execute before each command
pub(crate) fn pre_command(ctx: Context) -> BoxFuture<()> {
	Box::pin(async move {
		tracing::info!(
			user_id = ctx.author().id.0,
			"{} invoked {}",
			&ctx.author().name,
			ctx.invoked_command_name(),
		);
	})
}

/// Execute on a error during code execution
#[allow(clippy::too_many_lines)]
pub(crate) fn command_on_error(error: FrameworkError) -> BoxFuture<()> {
	Box::pin(async move {
		let error = match error {
			FrameworkError::Command { error, ctx, .. } => {
				let error_identifier = Uuid::new_v4().hyphenated().to_string();

				tracing::error!(
					user_id = ctx.author().id.0,
					error_id = error_identifier,
					"{} invoked `{}` but an error occurred: {:#}",
					ctx.author().name,
					ctx.invoked_command_name(),
					error
				);

				let error_msg = ctx.get(
					"error-internal-with-id",
					Some(&fluent_args!["id" => error_identifier]),
				);

				ctx.shout(error_msg)
					.await
					.map(|_| ())
					.context("Failed to send internal error message")
			}

			FrameworkError::CooldownHit {
				remaining_cooldown,
				ctx,
			} => {
				let content = ctx.get(
					"error-cooldown",
					Some(&fluent_args!["seconds" => remaining_cooldown.as_secs()]),
				);
				ctx.shout(content)
					.await
					.map(|_| ())
					.context("Failed to send cooldown hit message")
			}

			FrameworkError::MissingBotPermissions {
				ctx,
				missing_permissions,
			} => {
				let content = ctx.get(
					"error-bot-missing-permissions",
					Some(&fluent_args!["permissions" => missing_permissions.to_string()]),
				);
				ctx.shout(content)
					.await
					.map(|_| ())
					.context("Failed to send missing bot permissions message")
			}

			FrameworkError::MissingUserPermissions {
				ctx,
				missing_permissions,
			} => {
				let text = match missing_permissions {
					Some(permission) => ctx.get(
						"error-user-missing-permissions",
						Some(&fluent_args!["permissions" => permission.to_string()]),
					),
					None => ctx.get("error-user-missing-unknown-permissions", None),
				};

				ctx.shout(text)
					.await
					.map(|_| ())
					.context("Failed to send missing user permissions message")
			}

			FrameworkError::NotAnOwner { ctx } => {
				let content = ctx.get("error-not-an-owner", None);
				ctx.shout(content)
					.await
					.map(|_| ())
					.context("Failed to send not an owner message")
			}

			FrameworkError::GuildOnly { ctx } => {
				let content = ctx.get("error-guild-only", None);
				ctx.shout(content)
					.await
					.map(|_| ())
					.context("Failed to send guild only message")
			}

			FrameworkError::DmOnly { ctx } => {
				let content = ctx.get("error-dm-only", None);
				ctx.shout(content)
					.await
					.map(|_| ())
					.context("Failed to send dm only message")
			}

			FrameworkError::CommandCheckFailed { ctx, error } => {
				let error_identifier = Uuid::new_v4().hyphenated().to_string();

				tracing::error!(
					user_id = ctx.author().id.0,
					error_id = error_identifier,
					"{} invoked `{}` but an error occurred in command check: {:#}",
					ctx.author().name,
					ctx.invoked_command_name(),
					error.unwrap_or_else(|| anyhow!("Unknown error"))
				);

				let error_msg = ctx.get(
					"error-internal-with-id",
					Some(&fluent_args!["id" => error_identifier]),
				);

				ctx.shout(error_msg)
					.await
					.map(|_| ())
					.context("Failed to send command check failed message")
			}

			error => {
				tracing::error!(error = ?error, "Framework error");

				Ok(())
			}
		};

		if let Err(error) = error {
			tracing::error!(error = ?error);
		};
	})
}

/// Execute after every successful command
pub(crate) fn post_command(ctx: Context) -> BoxFuture<()> {
	Box::pin(async move {
		tracing::debug!(
			user_id = ctx.author().id.0,
			"{} invoked `{}` successfully!",
			&ctx.author().name,
			ctx.invoked_command_name(),
		);
	})
}
