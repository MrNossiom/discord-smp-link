//! `Discord` client events handlers

use std::sync::{atomic::AtomicBool, Arc};

use crate::{
	constants::events::{LOGIN_BUTTON_INTERACTION, LOGOUT_BUTTON_INTERACTION},
	database::{
		models::{Guild, Member, NewGuild, NewMember},
		schema::{guilds, members},
	},
	states::{Data, FrameworkContext, MessageComponentContext},
};
use anyhow::Result;
use diesel::prelude::*;
use poise::{
	serenity_prelude::{Context, Interaction},
	Event,
};

mod login;
mod logout;

/// Serenity listener to react to `Discord` events
pub(crate) async fn event_handler(
	ctx: &Context,
	event: &Event<'_>,
	framework: FrameworkContext<'_>,
	data: &Arc<Data>,
) -> Result<()> {
	match event {
		Event::Ready { data_about_bot } => {
			tracing::info!("`{}` is ready!", data_about_bot.user.name);

			Ok(())
		}

		Event::GuildMemberAddition { new_member } => {
			let mut connection = data.database.get()?;

			if let Ok(user) = members::table
				.filter(members::discord_id.eq(new_member.user.id.0))
				.filter(members::guild_id.eq(new_member.guild_id.0))
				.first::<Member>(&mut connection)
			{
				tracing::warn!(
					"User `{}` ({}) already exists in the database",
					user.username,
					user.discord_id
				);
			} else {
				let new_user = NewMember {
					guild_id: new_member.guild_id.0,
					username: new_member.user.name.as_str(),
					discord_id: new_member.user.id.0,
				};

				tracing::info!(
					"Adding user `{}` ({}) to database",
					new_user.username,
					new_user.discord_id
				);

				diesel::insert_into(members::table)
					.values(&new_user)
					.execute(&mut connection)?;
			}

			Ok(())
		}

		Event::GuildMemberRemoval { guild_id, user, .. } => {
			tracing::info!("Deleting member ({})", guild_id.0);

			diesel::delete(
				members::table
					.filter(members::guild_id.eq(guild_id.0))
					.filter(members::discord_id.eq(user.id.0)),
			)
			.execute(&mut data.database.get()?)?;

			Ok(())
		}

		Event::GuildCreate { guild, .. } => {
			let mut connection = data.database.get()?;

			if let Ok(guild) = guilds::table
				.filter(guilds::id.eq(guild.id.0))
				.first::<Guild>(&mut connection)
			{
				tracing::warn!(
					"Guild `{}` ({}) already exists in the database",
					guild.name,
					guild.id
				);
			} else {
				let new_guild = NewGuild {
					id: guild.id.0,
					name: guild.name.as_str(),
					owner_id: guild.owner_id.0,
					setup_message_id: None,
					verified_role_id: None,
				};

				tracing::info!("Adding guild `{}` ({}) to database", guild.name, guild.id);

				diesel::insert_into(guilds::table)
					.values(&new_guild)
					.execute(&mut connection)?;
			}

			Ok(())
		}

		Event::GuildDelete { incomplete, .. } => {
			tracing::warn!("Deleting guild ({})", incomplete.id);

			diesel::delete(guilds::table.filter(guilds::id.eq(incomplete.id.0)))
				.execute(&mut data.database.get()?)?;

			Ok(())
		}

		Event::InteractionCreate {
			interaction: Interaction::MessageComponent(interaction),
		} => {
			let ctx = MessageComponentContext {
				interaction,
				framework,
				data,
				discord: ctx,
				has_sent_initial_response: &AtomicBool::new(false),
			};

			tracing::info!(
				"`{}` interacted with a component `{}`",
				ctx.interaction.user.name,
				ctx.interaction.data.custom_id,
			);

			match interaction.data.custom_id.as_str() {
				LOGIN_BUTTON_INTERACTION => login::login(ctx).await,
				LOGOUT_BUTTON_INTERACTION => logout::logout(ctx).await,

				_ => Ok(()),
			}
		}

		_ => {
			tracing::trace!("Didn't handle event: {:?}", event);

			Ok(())
		}
	}
}
