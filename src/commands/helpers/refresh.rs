//! A set of commands to refresh the database

use crate::{
	database::models::{Member, NewMember},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use anyhow::anyhow;
use diesel::{
	prelude::*,
	result::{DatabaseErrorKind, Error as DieselError},
};
use fluent::fluent_args;
use poise::{command, serenity_prelude as serenity};

/// A set of commands to refresh the database
#[allow(clippy::unused_async)]
#[command(
	slash_command,
	owners_only,
	hide_in_help,
	subcommands("member", "members")
)]
pub(super) async fn refresh(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Loads a guild member in the database
#[command(slash_command, owners_only, hide_in_help)]
pub(super) async fn member(
	ctx: ApplicationContext<'_>,
	user: serenity::Member,
) -> InteractionResult {
	use crate::database::schema::members::dsl as members;

	let mut connection = ctx.data.database.get()?;

	if let Ok(member) = members::members
		.filter(members::discord_id.eq(user.user.id.0))
		.filter(members::guild_id.eq(user.guild_id.0))
		.first::<Member>(&mut connection)
	{
		let content = ctx.get(
			"debug-refresh-user-already-in-database",
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
			"debug-refresh-user-added",
			Some(&fluent_args!["user" => new_user.username]),
		);
		ctx.shout(content).await?;

		diesel::insert_into(members::members)
			.values(&new_user)
			.execute(&mut connection)?;
	}

	Ok(())
}

/// Loads every guild member in the database
#[command(slash_command, owners_only, hide_in_help)]
pub(super) async fn members(ctx: ApplicationContext<'_>) -> InteractionResult {
	use crate::database::schema::members::dsl as members;

	let mut connection = ctx.data.database.get()?;
	let guild_id = ctx
		.interaction
		.guild_id()
		.ok_or_else(|| anyhow!("guild only command"))?;

	let mut count = 0;
	let mut last_member_id = None;

	loop {
		let members = guild_id.members(ctx.discord, None, last_member_id).await?;
		let len = members.len();

		if let Some(member) = members.last() {
			last_member_id = Some(member.user.id);
		}

		for member in members {
			if member.user.bot {
				continue;
			}

			let new_user = NewMember {
				guild_id: member.guild_id.0,
				username: member.user.name.as_str(),
				discord_id: member.user.id.0,
			};

			match diesel::insert_into(members::members)
				.values(&new_user)
				.execute(&mut connection)
			{
				Ok(_) => count += 1,
				Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => continue,
				Err(error) => return Err(error.into()),
			};
		}

		if len < 1000 {
			break;
		}
	}

	let get = ctx.get(
		"debug-refresh-members-added",
		Some(&fluent_args!["count" => count]),
	);
	ctx.shout(get).await?;

	Ok(())
}
