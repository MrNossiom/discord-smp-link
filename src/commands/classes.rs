//! Setup messages for roles interactions

use crate::{
	database::{
		models::{Class, NewClass},
		prelude::*,
		schema,
	},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use anyhow::anyhow;
use diesel_async::RunQueryDsl;
use fluent::fluent_args;
use poise::{
	command,
	serenity_prelude::{self as serenity, Permissions},
};

// TODO: modify in the future
/// Add, modify or delete a class role.
#[allow(clippy::unused_async)]
#[command(
	slash_command,
	subcommands("classes_add", "classes_remove", "classes_list"),
	default_member_permissions = "MANAGE_ROLES"
)]
pub(crate) async fn classes(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Configure a new class role
#[command(slash_command, guild_only, rename = "add")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn classes_add(
	ctx: ApplicationContext<'_>,
	class_name: String,
	maybe_role: Option<serenity::Role>,
) -> InteractionResult {
	let guild_id = ctx
		.interaction
		.guild_id()
		.ok_or_else(|| anyhow!("guild only command"))?;

	let role = if let Some(role) = maybe_role {
		role
	} else {
		guild_id
			.create_role(ctx.discord, |role| {
				role.name(&class_name)
					.permissions(Permissions::empty())
					.mentionable(true)
			})
			.await?
	};

	let new_class = NewClass {
		guild_id: guild_id.0,
		role_id: role.id.0,
		name: &class_name,
	};

	new_class
		.insert()
		.execute(&mut ctx.data.database.get().await?)
		.await?;

	Ok(())
}

// TODO: optimize with a cache or whatever, db query intensive
/// The autocomplete function for the `class remove` name parameter.
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
async fn autocomplete_classes<'a>(ctx: ApplicationContext<'_>, partial: &'a str) -> Vec<String> {
	Class::all_from_guild(&ctx.interaction.guild_id().unwrap())
		.filter(schema::classes::name.like(format!("%{}%", partial)))
		.select(schema::classes::name)
		.get_results::<String>(&mut ctx.data.database.get().await.unwrap())
		.await
		.unwrap()
}

/// Delete a class role
#[command(slash_command, guild_only, rename = "remove")]
#[tracing::instrument(skip(_ctx))]
pub(crate) async fn classes_remove(
	_ctx: ApplicationContext<'_>,
	#[autocomplete = "autocomplete_classes"] class_name: String,
) -> InteractionResult {
	todo!();
}

/// List all the available class roles
#[command(slash_command, guild_only, rename = "list")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn classes_list(
	ctx: ApplicationContext<'_>,
	filter: Option<String>,
) -> InteractionResult {
	let guild_id = ctx
		.interaction
		.guild_id()
		.ok_or_else(|| anyhow!("guild only command"))?;

	// TODO: check role permissions

	let classes: Vec<String> = {
		let mut query = Class::all_from_guild(&guild_id)
			.select(schema::classes::name)
			.into_boxed();

		if let Some(ref filter) = filter {
			query = query.filter(schema::classes::name.like(format!("%{}%", filter)));
		};

		query
			.get_results::<String>(&mut ctx.data.database.get().await?)
			.await?
	};

	if classes.is_empty() {
		let get = if let Some(ref filter) = filter {
			ctx.get(
				"classes_list-none-with-filter",
				Some(&fluent_args!["filter" => filter.clone()]),
			)
		} else {
			ctx.get("classes_list-none", None)
		};
		ctx.shout(get).await?;

		return Ok(());
	}

	let classes_string = if classes.len() == 1 {
		format!("`{}`", classes[0])
	} else {
		format!(
			"`{}` {} `{}`",
			classes[..classes.len() - 1].join("`, `"),
			ctx.get("and", None),
			classes[classes.len() - 1]
		)
	};

	let message = format!(
		"**{}**:\n{}",
		if let Some(filter) = filter {
			ctx.get(
				"classes_list-title-with-filter",
				Some(&fluent_args!["filter" => filter]),
			)
		} else {
			ctx.get("classes_list-title", None)
		},
		classes_string
	);
	ctx.shout(message).await?;

	Ok(())
}
