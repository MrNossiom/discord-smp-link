//! Setup messages for roles interactions

use crate::{
	database::{prelude::*, schema},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use poise::{
	command,
	serenity_prelude::{self as serenity},
};

/// Setup the role to apply to verified members.
#[command(slash_command, guild_only, rename = "role")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn setup_role(
	ctx: ApplicationContext<'_>,
	role: serenity::Role,
) -> InteractionResult {
	let guild_id = ctx.guild_only_id();

	// TODO: check role permissions

	// Update the `verified_role_id`
	diesel::update(schema::guilds::table.find(guild_id.0))
		.set(schema::guilds::verified_role_id.eq(role.id.0))
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	let get = ctx.translate("done", None);
	ctx.shout(get).await?;

	Ok(())
}
