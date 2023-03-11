//! Setup messages for roles interactions

use crate::{
	constants,
	database::{models::Group, prelude::*, schema},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use poise::{
	command,
	serenity_prelude::{CreateSelectMenuOption, ReactionType},
};

/// Sets the login and logout message.
#[command(slash_command, guild_only, rename = "groups_message")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn setup_groups_message(ctx: ApplicationContext<'_>) -> InteractionResult {
	let mut connection = ctx.data.database.get().await?;
	let guild_id = ctx.guild_only_id();

	// TODO: check that there is a least one group
	// TODO: use guild locale or interaction locale as fallback

	let mut groups: Vec<Group> = Group::all_from_guild(guild_id)
		.load(&mut connection)
		.await?;

	if groups.is_empty() {
		let translate = ctx.translate("setup_groups_message-not-enough-groups", None);
		ctx.shout(translate).await?;

		return Ok(());
	}

	let groups = groups
		.iter_mut()
		.map(|group| {
			let mut op = CreateSelectMenuOption::new(&group.name, group.id);
			if let Some(emoji) = &group.emoji &&
				let Ok(emoji) = emoji.parse::<ReactionType>() {
				// TODO: check if the emoji is valid, even tough it should be checked before
				op.emoji(emoji);
			}
			op
		})
		.collect::<Vec<_>>();

	let reply = ctx
		.interaction
		.channel_id()
		.send_message(&ctx.serenity_context, |m| {
			m.content(ctx.translate("setup_groups_message-message", None))
				.components(|com| {
					com.create_action_row(|row| {
						row.create_select_menu(|select| {
							select
								.placeholder(
									ctx.translate("setup_groups_message-placeholder", None),
								)
								.options(|ops| ops.set_options(groups))
								.custom_id(constants::events::GROUPS_SELECT_MENU_INTERACTION)
						})
					})
				})
		})
		.await?;

	// Update the `setup_message_id`
	diesel::update(schema::guilds::table.find(guild_id.0))
		.set(schema::guilds::groups_message_id.eq(reply.id.0))
		.execute(&mut connection)
		.await?;

	let translate = ctx.translate("done", None);
	ctx.shout(translate).await?;

	Ok(())
}
