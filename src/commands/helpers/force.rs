//! A set of commands to force actions like login or logout

use crate::{
	database::{
		models::{Member, NewVerifiedMember, VerifiedMember},
		schema::{members, verified_members},
	},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use diesel::prelude::*;
use fluent::fluent_args;
use poise::{command, serenity_prelude as serenity};

/// A set of commands to force actions like login or logout
#[allow(clippy::unused_async)]
#[command(slash_command, owners_only, hide_in_help, subcommands("login"))]
pub(super) async fn force(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Force login a member
#[command(slash_command, owners_only, hide_in_help)]
pub(super) async fn login(
	ctx: ApplicationContext<'_>,
	user: serenity::Member,
) -> InteractionResult {
	let mut connection = ctx.data.database.get()?;

	if let Ok(member) = members::table
		.filter(members::discord_id.eq(user.user.id.0))
		.filter(members::guild_id.eq(user.guild_id.0))
		.first::<Member>(&mut connection)
	{
		match VerifiedMember::belonging_to(&member).first::<VerifiedMember>(&mut connection) {
			Ok(_) => {
				let content = ctx.get(
					"dev-force-login-already-verified",
					Some(&fluent_args!["user" => member.username]),
				);
				ctx.shout(content).await?;
			}
			Err(_) => {
				let new_verified_member = NewVerifiedMember {
					member_id: member.id,
					first_name: "John",
					last_name: "Doe",
					mail: "johndoe@gmail.com",
				};

				diesel::insert_into(verified_members::table)
					.values(&new_verified_member)
					.execute(&mut connection)?;

				let content = ctx.get(
					"dev-force-login-added",
					Some(&fluent_args!["user" => member.username]),
				);
				ctx.shout(content).await?;
			}
		}
	} else {
		let content = ctx.get(
			"dev-force-login-no-member",
			Some(&fluent_args!["user" => user.user.name]),
		);
		ctx.shout(content).await?;
	}

	Ok(())
}
