//! Setup messages for roles interactions

use crate::{
	database::{
		models::{Level, NewLevel},
		prelude::*,
		schema,
	},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use fluent::fluent_args;
use poise::{
	command,
	serenity_prelude::{self as serenity, Permissions, Role, RoleId},
};

/// TODO: possibility to modify a level
/// Add or delete a [`Level`]
#[allow(clippy::unused_async)]
#[command(
	slash_command,
	subcommands("levels_add", "levels_remove", "levels_list"),
	default_member_permissions = "MANAGE_ROLES",
	required_bot_permissions = "MANAGE_ROLES"
)]
pub(crate) async fn levels(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Configure a new level tag role
#[command(slash_command, guild_only, rename = "add")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn levels_add(
	ctx: ApplicationContext<'_>,
	name: String,
	role: Option<Role>,
) -> InteractionResult {
	let guild_id = ctx.guild_only_id();

	let role = match role {
		Some(role) => role,
		None => {
			guild_id
				.create_role(ctx.discord, |role| {
					role.name(&name)
						.permissions(Permissions::empty())
						.mentionable(true)
				})
				.await?
		}
	};

	let new_level = NewLevel {
		guild_id: guild_id.0,
		role_id: role.id.0,
		name: &name,
	};

	new_level
		.insert()
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	let translate = ctx.translate(
		"levels_add-success",
		Some(&fluent_args! { "level" => name }),
	);
	ctx.shout(translate).await?;

	Ok(())
}

// TODO: allow using a result instead of unwrapping everything
/// Autocompletes parameter for `levels` available in `Guild`.
#[allow(clippy::unwrap_used)]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(super) async fn autocomplete_levels<'a>(
	ctx: ApplicationContext<'_>,
	partial: &'a str,
) -> impl Iterator<Item = String> + 'a {
	// TODO: cache this per guild, db query intensive
	let levels: Vec<_> = Level::all_from_guild(&ctx.interaction.guild_id().unwrap())
		.select(schema::levels::name)
		.get_results::<String>(&mut ctx.data.database.get().await.unwrap())
		.await
		.unwrap();

	levels
		.into_iter()
		.filter(move |level| level.contains(partial))
}

/// Delete a level tag role
#[command(slash_command, guild_only, rename = "remove")]
#[tracing::instrument(skip(ctx))]
pub(crate) async fn levels_remove(
	ctx: ApplicationContext<'_>,
	#[autocomplete = "autocomplete_levels"] name: String,
) -> InteractionResult {
	let guild_id = ctx.guild_only_id();

	let (id, role_id) = match Level::all_from_guild(&guild_id)
		.filter(schema::levels::name.eq(&name))
		.select((schema::levels::id, schema::levels::role_id))
		.first::<(i32, u64)>(&mut ctx.data.database.get().await?)
		.await
	{
		Ok(tuple) => tuple,
		Err(DieselError::NotFound) => {
			let translate = ctx.translate("levels_remove-not-found", None);
			ctx.shout(translate).await?;
			return Ok(());
		}
		Err(err) => return Err(err.into()),
	};

	match guild_id.delete_role(ctx.discord, RoleId(role_id)).await {
		Ok(_) => {}
		// Ignore the error if the role is already deleted
		Err(serenity::Error::Http(_)) => {}
		Err(error) => return Err(error.into()),
	};

	diesel::delete(Level::with_id(id))
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	Ok(())
}

/// List all available level tag roles
#[command(slash_command, guild_only, rename = "list")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn levels_list(
	ctx: ApplicationContext<'_>,
	#[autocomplete = "autocomplete_levels"] filter: Option<String>,
) -> InteractionResult {
	let guild_id = ctx.guild_only_id();

	// TODO: use the cache from autocomplete context
	let levels: Vec<String> = Level::all_from_guild(&guild_id)
		.select(schema::levels::name)
		.get_results::<String>(&mut ctx.data.database.get().await?)
		.await?;

	let levels = match filter {
		None => levels,
		Some(ref predicate) => levels
			.into_iter()
			.filter(move |level| level.contains(predicate.as_str()))
			.collect(),
	};

	if levels.is_empty() {
		let get = if let Some(ref filter) = filter {
			ctx.translate(
				"levels_list-none-with-filter",
				Some(&fluent_args!["filter" => filter.clone()]),
			)
		} else {
			ctx.translate("levels_list-none", None)
		};
		ctx.shout(get).await?;

		return Ok(());
	}

	// TODO: show the classes that are under a certain level

	let levels_string = if levels.len() == 1 {
		format!("`{}`", levels[0])
	} else {
		format!(
			"`{}` {} `{}`",
			levels[..levels.len() - 1].join("`, `"),
			ctx.translate("and", None),
			levels[levels.len() - 1]
		)
	};

	let message = format!(
		"**{}**:\n{}",
		if let Some(filter) = filter {
			ctx.translate(
				"levels_list-title-with-filter",
				Some(&fluent_args!["filter" => filter]),
			)
		} else {
			ctx.translate("levels_list-title", None)
		},
		levels_string
	);
	ctx.shout(message).await?;

	Ok(())
}
