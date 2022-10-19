//! Setup messages for roles interactions

use crate::{
	database::{models::Class, schema},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use anyhow::anyhow;
use fluent::fluent_args;
use poise::command;

// TODO: modify in the future
/// Add, modify or delete a class role.
#[allow(clippy::unused_async)]
#[command(slash_command, subcommands("add", "remove", "list"))]
pub(crate) async fn class(_ctx: ApplicationContext<'_>) -> InteractionResult {
	Ok(())
}

/// Configure a new class role
#[command(slash_command, guild_only, default_member_permissions = "MANAGE_ROLES")]
pub(crate) async fn add(ctx: ApplicationContext<'_>, _class_name: String) -> InteractionResult {
	let _command_guild_id = ctx
		.interaction
		.guild_id()
		.ok_or_else(|| anyhow!("guild only command"))?;

	Ok(())
}

// TODO: optimize with a cache or whatever, db query intensive
/// The autocomplete function for the `class remove` name parameter.
async fn autocomplete_classes<'a>(ctx: ApplicationContext<'_>, partial: &'a str) -> Vec<String> {
	use diesel::prelude::*;

	Class::all_from_guild(&ctx.interaction.guild_id().unwrap())
		.filter(schema::classes::name.like(format!("%{}%", partial)))
		.select(schema::classes::name)
		.get_results::<String>(&mut ctx.data.database.get().unwrap())
		.unwrap()
}

/// Delete a class role
#[command(slash_command, guild_only, default_member_permissions = "MANAGE_ROLES")]
pub(crate) async fn remove(
	_ctx: ApplicationContext<'_>,
	#[autocomplete = "autocomplete_classes"] _class_name: String,
) -> InteractionResult {
	todo!();
}

/// List all the available class roles
#[command(slash_command, guild_only, default_member_permissions = "MANAGE_ROLES")]
pub(crate) async fn list(ctx: ApplicationContext<'_>, filter: Option<String>) -> InteractionResult {
	let guild_id = ctx
		.interaction
		.guild_id()
		.ok_or_else(|| anyhow!("guild only command"))?;

	// TODO: check role permissions

	let classes: Vec<String> = {
		use diesel::prelude::*;

		let mut query = Class::all_from_guild(&guild_id)
			.select(schema::classes::name)
			.into_boxed();

		if let Some(ref filter) = filter {
			query = query.filter(schema::classes::name.like(format!("%{}%", filter)));
		};

		query.get_results::<String>(&mut ctx.data.database.get()?)?
	};

	if classes.is_empty() {
		let get = if let Some(ref filter) = filter {
			ctx.get(
				"class-list-none-with-filter",
				Some(&fluent_args!["filter" => filter.clone()]),
			)
		} else {
			ctx.get("class-list-none", None)
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
				"class-list-title-with-filter",
				Some(&fluent_args!["filter" => filter]),
			)
		} else {
			ctx.get("class-list-title", None)
		},
		classes_string
	);
	ctx.shout(message).await?;

	Ok(())
}
