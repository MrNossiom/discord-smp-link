//! A set of commands to force actions like login or logout

use crate::{
	database::schema::{members::dsl as members, verified_members::dsl as verified_members},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use diesel::prelude::*;
use fluent::fluent_args;
use poise::{command, serenity_prelude as serenity};

/// A set of commands to force actions like login or logout
#[allow(clippy::unused_async)]
#[command(slash_command, owners_only, hide_in_help, subcommands("logout"))]
pub(super) async fn force(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Force logout a member
#[command(slash_command, owners_only, hide_in_help)]
pub(super) async fn logout(
	ctx: ApplicationContext<'_>,
	user: serenity::Member,
) -> InteractionResult {
	let mut connection = ctx.data.database.get()?;

	if let Ok(member_id) = verified_members::verified_members
		.inner_join(members::members)
		.filter(members::discord_id.eq(user.user.id.0))
		.filter(members::guild_id.eq(user.guild_id.0))
		.select(verified_members::member_id)
		.first::<i32>(&mut connection)
	{
		diesel::delete(
			verified_members::verified_members.filter(verified_members::member_id.eq(member_id)),
		)
		.execute(&mut connection)?;

		let content = ctx.get(
			"debug-force-logout-done",
			Some(&fluent_args!["user" => user.user.name]),
		);
		ctx.shout(content).await?;
	} else {
		let content = ctx.get(
			"error-member-not-verified",
			Some(&fluent_args!["user" => user.user.name]),
		);
		ctx.shout(content).await?;
	}

	Ok(())
}
