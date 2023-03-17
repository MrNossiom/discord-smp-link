//! A set of commands to manage the bot.

use crate::{
	database::{models::Guild, prelude::*, schema},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use fluent::fluent_args;
use poise::{
	command,
	serenity_prelude::{Permissions, Role},
};

mod groups_message;
mod login_message;

use groups_message::setup_groups_message;
use login_message::setup_login_message;

/// A set of commands to setup the bot
#[allow(clippy::unused_async)]
#[command(
	slash_command,
	rename = "setup",
	subcommands(
		"setup_groups_message",
		"setup_login_message",
		"setup_role",
		"setup_pattern"
	),
	default_member_permissions = "ADMINISTRATOR"
)]
pub(crate) async fn setup(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Setup the role to apply to verified members.
#[command(slash_command, guild_only, rename = "role")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn setup_role(ctx: ApplicationContext<'_>, role: Role) -> InteractionResult {
	let guild_id = ctx.guild_only_id();

	if role.has_permission(Permissions::ADMINISTRATOR) {
		ctx.shout(ctx.translate("setup_role-role-admin", None))
			.await?;

		return Ok(());
	}

	// Update the verified role
	diesel::update(Guild::with_id(guild_id))
		.set(schema::guilds::verified_role_id.eq(role.id.0))
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	ctx.shout(ctx.translate("done", None)).await?;

	Ok(())
}

/// Setup the role to apply to verified members.
#[command(slash_command, guild_only, rename = "pattern")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn setup_pattern(
	ctx: ApplicationContext<'_>,
	pattern: String,
) -> InteractionResult {
	let guild_id = ctx.guild_only_id();

	// Update the verification email domain
	diesel::update(Guild::with_id(guild_id))
		.set(schema::guilds::verification_email_domain.eq(&pattern))
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	ctx.shout(ctx.translate(
		"setup_pattern-done",
		Some(fluent_args!["pattern" => pattern]),
	))
	.await?;

	Ok(())
}
