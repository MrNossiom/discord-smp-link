//! Setup messages for roles interactions

use crate::{
	constants,
	database::{models::Group, prelude::*, schema},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use poise::{
	command,
	serenity_prelude::{
		CreateActionRow, CreateMessage, CreateSelectMenu, CreateSelectMenuKind,
		CreateSelectMenuOption, ReactionType,
	},
};

/// Sets the login and logout message.
#[command(slash_command, guild_only, rename = "groups_message")]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user.id))]
pub(crate) async fn setup_groups_message(ctx: ApplicationContext<'_>) -> InteractionResult {
	let mut connection = ctx.data.database.get().await?;
	let guild_id = ctx.guild_only_id();

	// TODO: check that there is a least one group
	// TODO: use guild locale or interaction locale as fallback

	let mut groups: Vec<Group> = Group::all_from_guild(guild_id)
		.load(&mut connection)
		.await?;

	if groups.is_empty() {
		ctx.shout(ctx.translate("setup_groups_message-not-enough-groups", None))
			.await?;

		return Ok(());
	}

	let groups = groups
		.iter_mut()
		.map(|group| {
			let mut op = CreateSelectMenuOption::new(&group.name, group.id.to_string());
			if let Some(emoji) = &group.emoji {
				if let Ok(emoji) = emoji.parse::<ReactionType>() {
					// TODO: check if the emoji is valid, even tough it should be checked before
					op = op.emoji(emoji);
				}
			}
			op
		})
		.collect::<Vec<_>>();

	let action_row = CreateActionRow::SelectMenu(
		CreateSelectMenu::new(
			constants::events::GROUPS_SELECT_MENU_INTERACTION,
			CreateSelectMenuKind::String { options: groups },
		)
		.placeholder(ctx.translate("setup_groups_message-placeholder", None)),
	);

	let reply = ctx
		.interaction
		.channel_id
		.send_message(
			&ctx.serenity_context,
			CreateMessage::new()
				.content(ctx.translate("setup_groups_message-message", None))
				.components(vec![action_row]),
		)
		.await?;

	// Update the `setup_message_id`
	diesel::update(schema::guilds::table.find(guild_id.get()))
		.set(schema::guilds::groups_message_id.eq(reply.id.get()))
		.execute(&mut connection)
		.await?;

	ctx.shout(ctx.translate("done", None)).await?;

	Ok(())
}
