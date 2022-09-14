//! Setup messages for roles interactions

use crate::{
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use anyhow::anyhow;
use poise::{
	command,
	serenity_prelude::{self as serenity},
};

/// Setup the role to apply to verified members.
#[command(
	slash_command,
	guild_only,
	default_member_permissions = "ADMINISTRATOR"
)]
pub(crate) async fn role(ctx: ApplicationContext<'_>, role: serenity::Role) -> InteractionResult {
	let guild_id = ctx
		.interaction
		.guild_id()
		.ok_or_else(|| anyhow!("guild only command"))?;

	// TODO: check role permissions

	// Update the `verified_role_id`
	{
		use crate::database::schema::guilds::dsl::{guilds, verified_role_id};
		use diesel::prelude::*;

		diesel::update(guilds.find(guild_id.0))
			.set(verified_role_id.eq(role.id.0))
			.execute(&mut ctx.data.database.get()?)?;
	}

	let get = ctx.get("done", None);
	ctx.shout(get).await?;

	Ok(())
}
