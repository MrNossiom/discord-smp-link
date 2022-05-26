//! `Discord` client commands

use crate::states::Context;
use poise::BoxFuture;

pub mod helpers;
pub mod login;

/// Logs every command call
pub fn pre_command(ctx: Context) -> BoxFuture<'_, ()> {
	Box::pin(async move {
		log::info!(
			"{} invoked by {}",
			ctx.invoked_command_name(),
			ctx.author().name,
		);
	})
}
