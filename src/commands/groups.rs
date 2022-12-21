//! Setup messages for roles interactions

use crate::{
	constants,
	database::{
		models::{Group, NewGroup},
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

// TODO: possibility to modify a group
/// Add or delete a [`Group`]
#[allow(clippy::unused_async)]
#[command(
	slash_command,
	subcommands("groups_add", "groups_remove", "groups_list"),
	default_member_permissions = "MANAGE_ROLES",
	required_bot_permissions = "MANAGE_ROLES"
)]
pub(crate) async fn groups(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Configure a new group tag role
#[command(slash_command, guild_only, rename = "add")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn groups_add(
	ctx: ApplicationContext<'_>,
	name: String,
	role: Option<Role>,
) -> InteractionResult {
	let guild_id = ctx.guild_only_id();

	let role = match role {
		Some(role) => role,
		None => {
			guild_id
				.create_role(&ctx.serenity_context, |role| {
					role.name(&name)
						.permissions(Permissions::empty())
						.mentionable(true)
				})
				.await?
		}
	};

	let new_group = NewGroup {
		guild_id: guild_id.0,
		role_id: role.id.0,
		name: &name,
	};

	new_group
		.insert()
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	let translate = ctx.translate(
		"groups_add-success",
		Some(&fluent_args! { "group" => name }),
	);
	ctx.shout(translate).await?;

	Ok(())
}

// TODO: allow using a result instead of unwrapping everything
/// Autocompletes parameter for `groups` available in `Guild`.
#[allow(clippy::unwrap_used)]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
async fn autocomplete_groups<'a>(
	ctx: ApplicationContext<'_>,
	partial: &'a str,
) -> impl Iterator<Item = String> + 'a {
	// TODO: cache this per guild, db query intensive
	let groups: Vec<_> = Group::all_from_guild(ctx.interaction.guild_id().unwrap())
		.select(schema::groups::name)
		.get_results::<String>(&mut ctx.data.database.get().await.unwrap())
		.await
		.unwrap();

	groups
		.into_iter()
		.filter(move |group| group.contains(partial))
}

/// Delete a group tag role
#[command(slash_command, guild_only, rename = "remove")]
#[tracing::instrument(skip(ctx))]
pub(crate) async fn groups_remove(
	ctx: ApplicationContext<'_>,
	#[autocomplete = "autocomplete_groups"] name: String,
) -> InteractionResult {
	let guild_id = ctx.guild_only_id();

	let (id, role_id) = match Group::all_from_guild(guild_id)
		.filter(schema::groups::name.eq(&name))
		.select((schema::groups::id, schema::groups::role_id))
		.first::<(i32, u64)>(&mut ctx.data.database.get().await?)
		.await
	{
		Ok(tuple) => tuple,
		Err(DieselError::NotFound) => {
			let translate = ctx.translate("groups_remove-not-found", None);
			ctx.shout(translate).await?;
			return Ok(());
		}
		Err(err) => return Err(err.into()),
	};

	match guild_id
		.delete_role(&ctx.serenity_context, RoleId(role_id))
		.await
	{
		Ok(_) |
		// Ignore the error if the role is already deleted
		Err(serenity::Error::Http(_)) => {}
		Err(error) => return Err(error.into()),
	};

	diesel::delete(Group::with_id(id))
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	Ok(())
}

/// List all available group tag roles
#[command(slash_command, guild_only, rename = "list")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn groups_list(
	ctx: ApplicationContext<'_>,
	#[autocomplete = "autocomplete_groups"] filter: Option<String>,
) -> InteractionResult {
	let guild_id = ctx.guild_only_id();
	let mut connection = ctx.data.database.get().await?;

	let nb_of_groups: i64 = Group::all_from_guild(guild_id)
		.count()
		.get_result(&mut connection)
		.await?;

	if nb_of_groups >= i64::from(constants::limits::MAX_GROUPS_PER_GUILD) {
		let translate = ctx.translate("groups_add-too-many-groups", None);
		ctx.shout(translate).await?;

		return Ok(());
	}

	// TODO: use the cache from autocomplete context
	let groups: Vec<String> = Group::all_from_guild(guild_id)
		.select(schema::groups::name)
		.get_results::<String>(&mut connection)
		.await?;

	let groups = match filter {
		None => groups,
		Some(ref predicate) => groups
			.into_iter()
			.filter(move |group| group.contains(predicate.as_str()))
			.collect(),
	};

	if groups.is_empty() {
		let get = filter.as_ref().map_or_else(
			|| ctx.translate("groups_list-none", None),
			|filter| {
				ctx.translate(
					"groups_list-none-with-filter",
					Some(&fluent_args!["filter" => filter.clone()]),
				)
			},
		);
		ctx.shout(get).await?;

		return Ok(());
	}

	let groups_string = if groups.len() == 1 {
		format!("`{}`", groups[0])
	} else {
		format!(
			"`{}` {} `{}`",
			groups[..groups.len() - 1].join("`, `"),
			ctx.translate("and", None),
			groups[groups.len() - 1]
		)
	};

	let message = format!(
		"**{}**:\n{}",
		filter.map_or_else(
			|| ctx.translate("groups_list-title", None),
			|filter| ctx.translate(
				"groups_list-title-with-filter",
				Some(&fluent_args!["filter" => filter]),
			)
		),
		groups_string
	);
	ctx.shout(message).await?;

	Ok(())
}
