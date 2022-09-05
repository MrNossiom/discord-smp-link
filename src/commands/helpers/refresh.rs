//! A set of commands to refresh the database

use crate::{
	database::{
		models::{Member, NewMember},
		schema::members,
	},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use diesel::prelude::*;
use fluent::fluent_args;
use poise::{command, serenity_prelude as serenity};

/// A set of commands to refresh the database
#[allow(clippy::unused_async)]
#[command(slash_command, owners_only, hide_in_help, subcommands("user"))]
pub(super) async fn refresh(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Loads a user in the database
#[command(slash_command, owners_only, hide_in_help)]
pub(super) async fn user(ctx: ApplicationContext<'_>, user: serenity::Member) -> InteractionResult {
	let mut connection = ctx.data.database.get()?;

	if let Ok(member) = members::table
		.filter(members::discord_id.eq(user.user.id.0))
		.filter(members::guild_id.eq(user.guild_id.0))
		.first::<Member>(&mut connection)
	{
		let content = ctx.get(
			"dev-refresh-user-already-in-database",
			Some(&fluent_args!["user" => member.username]),
		);
		ctx.shout(content).await?;
	} else {
		let new_user = NewMember {
			guild_id: user.guild_id.0,
			username: user.user.name.as_str(),
			discord_id: user.user.id.0,
		};

		let content = ctx.get(
			"dev-refresh-user-added",
			Some(&fluent_args!["user" => new_user.username]),
		);
		ctx.shout(content).await?;

		diesel::insert_into(members::table)
			.values(&new_user)
			.execute(&mut connection)?;
	}

	Ok(())
}
