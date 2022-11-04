//! Command to link Discord and Google accounts together.

use crate::{
	auth::GoogleAuthentificationError,
	constants,
	database::{
		models::{Class, Guild, Level, Member, NewVerifiedMember},
		prelude::*,
		schema, DatabasePooledConnection,
	},
	states::{InteractionResult, MessageComponentContext},
	translation::Translate,
};
use anyhow::{anyhow, Context};
use fluent::fluent_args;
use poise::serenity_prelude::{
	self as serenity, component::ButtonStyle, CollectComponentInteraction, CreateSelectMenu,
	CreateSelectMenuOption, GuildId, RoleId,
};
use std::time::Duration;
use thiserror::Error;

// TODO: heist all requirements and move every database update or Discord call to the end
// TODO: document steps because it becomes messy here
/// Starts the auth process after the user clicked on the login button
#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, fields(caller_id = %ctx.interaction.user.id))]
pub(crate) async fn login(ctx: MessageComponentContext<'_>) -> InteractionResult {
	let mut connection = ctx.data.database.get().await?;
	let mut member = ctx.guild_only_member();

	let (verified_role, mut levels, email_pattern) =
		match check_login_components(&mut connection, &member.guild_id).await {
			Ok(v) => v,
			Err(err) => match err {
				CheckLoginComponentsError::Database(err) => return Err(err.into()),
				CheckLoginComponentsError::GuildNotTotallySetup(err) => {
					ctx.shout(err).await?;

					return Ok(());
				}
			},
		};

	let (oauth2_url, token_response) = ctx
		.data
		.auth
		.process_oauth2(constants::AUTHENTICATION_TIMEOUT);

	let initial_response = ctx
		.send(|reply| {
			reply
				.ephemeral(true)
				.content(ctx.translate("use-google-account-to-login", None))
				.components(|components| {
					components.create_action_row(|action_row| {
						action_row.create_button(|button| {
							button
								.label(ctx.translate("continue", None))
								.style(ButtonStyle::Link)
								.url(oauth2_url)
						})
					})
				})
		})
		.await?;

	let token_response = match token_response.await {
		Ok(response) => response,
		Err(GoogleAuthentificationError::Timeout) => {
			let content = ctx.translate("did-not-finish-auth-process", None);
			ctx.shout(content).await?;

			return Ok(());
		}
		Err(error) => return Err(error.into()),
	};

	let user_data = ctx
		.data
		.auth
		.query_google_user_metadata(&token_response)
		.await
		.context("Failed to query google user metadata")?;

	let mail_domain = user_data
		.mail
		.split('@')
		.last()
		.context("email returned by google is invalid")?;
	if mail_domain != email_pattern {
		let content = ctx.translate("event-login-email-domain-not-allowed", None);
		ctx.shout(content).await?;

		return Ok(());
	}

	let levels = levels
		.iter_mut()
		.map(|cl| CreateSelectMenuOption::new(&cl.name, cl.id))
		.collect::<Vec<_>>();

	let mut levels_select_menu = CreateSelectMenu::default();

	levels_select_menu
		.custom_id(constants::events::AUTHENTICATION_SELECT_MENU_INTERACTION)
		// TODO: translate
		.placeholder(ctx.translate("event-login-select-level", None))
		.options(|op| op.set_options(levels));

	initial_response
		.edit(|b| {
			b.ephemeral(true)
				.components(|c| c.create_action_row(|ar| ar.add_select_menu(levels_select_menu)))
				// Empty the previous content
				.content("")
		})
		.await?;

	let level_id = match CollectComponentInteraction::new(ctx.discord)
		.message_id(initial_response.message().await?.id)
		.timeout(Duration::from_secs(60))
		.await
	{
		Some(interaction) => {
			interaction.defer(ctx.discord).await?;

			interaction
				.data
				.values
				.get(0)
				.ok_or_else(|| anyhow!("Something went wrong while parsing class id"))?
				.parse::<i32>()?
		}
		None => {
			let content = ctx.translate("error-user-timeout", None);
			ctx.shout(content).await?;

			return Ok(());
		}
	};

	let mut classes: Vec<Class> = Class::all_from_level(level_id)
		.get_results::<Class>(&mut connection)
		.await?;

	let classes = classes
		.iter_mut()
		.map(|cl| CreateSelectMenuOption::new(&cl.name, cl.id))
		.collect::<Vec<_>>();

	let mut classes_select_menu = CreateSelectMenu::default();

	classes_select_menu
		.custom_id(constants::events::AUTHENTICATION_SELECT_MENU_INTERACTION)
		.placeholder(ctx.translate("event-login-select-class", None))
		.options(|op| op.set_options(classes));

	initial_response
		.edit(|b| {
			b.ephemeral(true)
				.components(|c| c.create_action_row(|ar| ar.add_select_menu(classes_select_menu)))
				// Empty the previous content
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
				.get(0)
				.ok_or_else(|| anyhow!("Something went wrong while parsing class id"))?
				.parse::<i32>()?
		}
		None => {
			let content = ctx.translate("error-user-timeout", None);
			ctx.shout(content).await?;

			return Ok(());
		}
	};

	let id = match Member::with_ids(&member.user.id, &member.guild_id)
		.select(schema::members::id)
		.first::<i32>(&mut connection)
		.await
	{
		Ok(id) => id,
		Err(DieselError::NotFound) => {
			let content = ctx.translate(
				"error-member-not-registered",
				Some(&fluent_args!["user" => member.user.name.as_str()]),
			);
			ctx.shout(content).await?;

			return Ok(());
		}
		Err(error) => return Err(error.into()),
	};

	let new_verified_member = NewVerifiedMember {
		member_id: id,
		first_name: &user_data.first_name,
		last_name: &user_data.last_name,
		mail: &user_data.mail,
		class_id,
	};

	new_verified_member
		.insert()
		.execute(&mut connection)
		.await?;

	let level_role = {
		let id = Level::with_id(level_id)
			.select(schema::levels::role_id)
			.first::<u64>(&mut connection)
			.await?;

		RoleId(id)
	};

	let class_role = {
		let id = Class::with_id(class_id)
			.select(schema::classes::role_id)
			.first::<u64>(&mut connection)
			.await?;

		RoleId(id)
	};

	match member.add_role(ctx.discord, verified_role).await {
		Ok(_) => {}
		Err(serenity::Error::Model(serenity::ModelError::RoleNotFound)) => {
			diesel::update(Guild::with_id(&member.guild_id))
				.set(schema::guilds::verified_role_id.eq::<Option<u64>>(None))
				.execute(&mut connection)
				.await?;
		}
		Err(error) => return Err(error.into()),
	}

	match member.add_role(ctx.discord, level_role).await {
		Ok(_) => {}
		// TODO: handle role deleted
		Err(error) => return Err(error.into()),
	}

	match member.add_role(ctx.discord, class_role).await {
		Ok(_) => {}
		// TODO: handle role deleted
		Err(error) => return Err(error.into()),
	}

	initial_response
		.edit(|b| {
			b.content(ctx.translate("authentication-successful", None))
				.components(|c| c)
		})
		.await?;

	Ok(())
}

// TODO: improve next function and remove this
/// Error type for the following function
#[derive(Debug, Error)]
enum CheckLoginComponentsError {
	/// An error to show to the user
	#[error("{0}")]
	GuildNotTotallySetup(String),

	/// An error from the database
	#[error(transparent)]
	Database(#[from] DieselError),
}

/// Extracted logic
async fn check_login_components(
	connection: &mut DatabasePooledConnection,
	guild_id: &GuildId,
) -> Result<(RoleId, Vec<Level>, String), CheckLoginComponentsError> {
	let (verified_role, email_pattern) = {
		let (inner_role_id, email_pattern): (Option<u64>, Option<String>) =
			Guild::with_id(guild_id)
				.select((
					schema::guilds::verified_role_id,
					schema::guilds::verification_email_domain,
				))
				.first(connection)
				.await?;

		let inner_role = match inner_role_id {
			Some(role_id) => RoleId(role_id),
			None => {
				// TODO: translate
				return Err(CheckLoginComponentsError::GuildNotTotallySetup(
					"Verified role has not been setup yet".into(),
				));
			}
		};

		let email_pattern = match email_pattern {
			Some(role) => role,
			None => {
				// TODO: translate
				return Err(CheckLoginComponentsError::GuildNotTotallySetup(
					"Email pattern has not been setup yet".into(),
				));
			}
		};

		(inner_role, email_pattern)
	};

	let levels: Vec<Level> = Level::all_from_guild(guild_id)
		.get_results::<Level>(connection)
		.await?;

	if levels.is_empty() {
		// TODO: translate
		return Err(CheckLoginComponentsError::GuildNotTotallySetup(
			"No levels found".into(),
		));
	}

	Ok((verified_role, levels, email_pattern))
}
