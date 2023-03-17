//! A set of commands to force actions like login or logout

use crate::{
	database::{
		prelude::*,
		schema::{members, verified_members},
	},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use fluent::fluent_args;
use poise::{command, serenity_prelude as serenity};

/// A set of commands to force actions like login or logout
#[allow(clippy::unused_async)]
#[command(
	slash_command,
	owners_only,
	hide_in_help,
	rename = "force",
	subcommands("debug_force_logout")
)]
pub(super) async fn debug_force(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Force logout a member
#[command(slash_command, owners_only, hide_in_help, rename = "logout")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(super) async fn debug_force_logout(
	ctx: ApplicationContext<'_>,
	user: serenity::Member,
) -> InteractionResult {
	let mut connection = ctx.data.database.get().await?;

	if let Ok(member_id) = verified_members::table
		.inner_join(members::table)
		.filter(members::discord_id.eq(user.user.id.0))
		.filter(members::guild_id.eq(user.guild_id.0))
		.select(verified_members::member_id)
		.first::<i32>(&mut connection)
		.await
	{
		diesel::delete(verified_members::table.filter(verified_members::member_id.eq(member_id)))
			.execute(&mut connection)
			.await?;

		ctx.shout(ctx.translate(
			"debug_force_logout-done",
			Some(fluent_args!["user" => user.user.name]),
		))
		.await?;
	} else {
		ctx.shout(ctx.translate(
			"error-member-not-verified",
			Some(fluent_args!["user" => user.user.name]),
		))
		.await?;
	}

	Ok(())
}
