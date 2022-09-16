//! Command to link Discord and Google accounts together.

use crate::{
	constants::{self, AUTHENTICATION_TIMEOUT},
	database::{
		models::{Class, NewVerifiedMember},
		DieselError,
	},
	handlers::auth::AuthProcessError,
	states::{InteractionResult, MessageComponentContext},
	translation::Translate,
};
use anyhow::{anyhow, Context};
use diesel::{
	prelude::*,
	r2d2::{ConnectionManager, PooledConnection},
};
use fluent::fluent_args;
use poise::serenity_prelude::{
	self as serenity, component::ButtonStyle, CollectComponentInteraction, CreateSelectMenu,
	CreateSelectMenuOption, GuildId, RoleId,
};
use std::time::Duration;

// TODO: heist all requirements and move every database or Discord call to the end
/// Starts the auth process after the user clicked on the login button
#[allow(clippy::too_many_lines)]
pub(crate) async fn login(ctx: MessageComponentContext<'_>) -> InteractionResult {
	let mut connection = ctx.data.database.get()?;
	let member = ctx
		.interaction
		.member
		.as_ref()
		.ok_or_else(|| anyhow!("used only in guild"))?;

	let role = {
		use crate::database::schema::guilds;

		let inner_role_id: Option<u64> = guilds::table
			.filter(guilds::id.eq(member.guild_id.0))
			.select(guilds::verified_role_id)
			.first(&mut connection)?;

		inner_role_id.map(RoleId)
	};

	let verified_role = match role {
		Some(role) => role,
		None => {
			ctx.shout("Verified role has not been setup yet").await?;

			return Ok(());
		}
	};

	let (oauth2_url, token_response) = ctx.data.auth.process_oauth2(AUTHENTICATION_TIMEOUT);

	let classes_select_menu = get_guild_classes_select_menu(&member.guild_id, &mut connection)?;

	let initial_response = ctx
		.send(|reply| {
			reply
				.ephemeral(true)
				.content(ctx.get("use-google-account-to-login", None))
				.components(|components| {
					components.create_action_row(|action_row| {
						action_row.create_button(|button| {
							button
								.label(ctx.get("continue", None))
								.style(ButtonStyle::Link)
								.url(oauth2_url)
						})
					})
				})
		})
		.await?;

	let token_response = match token_response.await {
		Ok(response) => response,
		Err(AuthProcessError::Timeout) => {
			let content = ctx.get("did-not-finish-auth-process", None);
			ctx.shout(content).await?;

			return Ok(());
		}
		Err(error) => return Err(error.into()),
	};

	initial_response
		.edit(|b| {
			b.ephemeral(true)
				.components(|c| c.create_action_row(|ar| ar.add_select_menu(classes_select_menu)))
				// Empty the content
				.content("")
		})
		.await?;

	let class_id = match CollectComponentInteraction::new(ctx.discord)
		.message_id(initial_response.message().await?.id)
		.timeout(Duration::from_secs(60))
		.await
	{
		Some(interaction) => {
			interaction.defer(ctx.discord).await?;

			interaction
				.data
				.values
				.first()
				.ok_or_else(|| anyhow!("Something went wrong while parsing class id"))?
				.parse::<i32>()?
		}
		None => {
			let content = ctx.get("error-user-timeout", None);
			ctx.shout(content).await?;

			return Ok(());
		}
	};

	let user_data = ctx
		.data
		.auth
		.query_google_user_metadata(&token_response)
		.await
		.context("Failed to query google user metadata")?;

	let id = {
		use crate::database::schema::members::dsl as members;

		match members::members
			.filter(members::discord_id.eq(member.user.id.0))
			.filter(members::guild_id.eq(member.guild_id.0))
			.select(members::id)
			.first::<i32>(&mut ctx.data.database.get()?)
		{
			Ok(id) => id,
			Err(DieselError::NotFound) => {
				let content = ctx.get(
					"error-member-not-registered",
					Some(&fluent_args!["user" => member.user.name.as_str()]),
				);
				ctx.shout(content).await?;

				return Ok(());
			}
			Err(error) => return Err(error.into()),
		}
	};

	let new_verified_member = NewVerifiedMember {
		member_id: id,
		first_name: &user_data.first_name,
		last_name: &user_data.last_name,
		mail: &user_data.mail,
		class_id,
	};

	{
		use crate::database::schema::verified_members::dsl as verified_members;

		diesel::insert_into(verified_members::verified_members)
			.values(new_verified_member)
			.execute(&mut connection)?;
	}

	match member.clone().add_role(ctx.discord, verified_role).await {
		Ok(_) => {}

		Err(serenity::Error::Model(serenity::ModelError::RoleNotFound)) => {
			use crate::database::schema::guilds::dsl as guilds;

			diesel::update(guilds::guilds.filter(guilds::id.eq(member.guild_id.0)))
				.set(guilds::verified_role_id.eq::<Option<u64>>(None))
				.execute(&mut connection)?;
		}

		Err(error) => return Err(error.into()),
	}

	initial_response
		.edit(|b| b.content(ctx.get("authentication-successful", None)))
		.await?;

	Ok(())
}

///
fn get_guild_classes_select_menu(
	guild_id: &GuildId,
	connection: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
) -> anyhow::Result<CreateSelectMenu> {
	use crate::database::schema::classes::dsl as classes;

	let mut classes = classes::classes
		.filter(classes::guild_id.eq(guild_id.0))
		.get_results::<Class>(connection)?;

	if classes.is_empty() {
		// TODO: real handling
		return Err(anyhow!("No classes found"));
	}

	let classes = classes
		.iter_mut()
		.map(|cl| CreateSelectMenuOption::new(&cl.name, cl.id))
		.collect::<Vec<_>>();

	let mut sel = CreateSelectMenu::default();

	sel.custom_id(constants::events::AUTHENTICATION_SELECT_MENU_INTERACTION)
		// TODO: translate
		.placeholder("Select a class")
		.options(|op| op.set_options(classes));

	Ok(sel)
}
