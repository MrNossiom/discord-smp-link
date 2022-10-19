//! Command to link Discord and Google accounts together.

use crate::{
	auth::GoogleAuthentificationError,
	constants,
	database::{
		models::{Class, Guild, Member, NewVerifiedMember},
		schema, DatabasePooledConnection, DieselError,
	},
	states::{InteractionResult, MessageComponentContext},
	translation::Translate,
};
use anyhow::{anyhow, Context};
use diesel::prelude::*;
use fluent::fluent_args;
use poise::serenity_prelude::{
	self as serenity, component::ButtonStyle, CollectComponentInteraction, CreateSelectMenu,
	CreateSelectMenuOption, GuildId, RoleId,
};
use std::time::Duration;
use thiserror::Error;
use tracing::instrument;

// TODO: heist all requirements and move every database or Discord call to the end
// TODO: document steps because it becomes messy here
/// Starts the auth process after the user clicked on the login button
#[allow(clippy::too_many_lines)]
#[instrument(skip_all, fields(user_id = %ctx.interaction.user.id))]
pub(crate) async fn login(ctx: MessageComponentContext<'_>) -> InteractionResult {
	let mut connection = ctx.data.database.get()?;
	let member = ctx
		.interaction
		.member
		.as_ref()
		.ok_or_else(|| anyhow!("used only in guild"))?;

	let (verified_role, mut classes, email_pattern) =
		match get_and_check_login_components(&mut connection, &member.guild_id) {
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

	let classes = classes
		.iter_mut()
		.map(|cl| CreateSelectMenuOption::new(&cl.name, cl.id))
		.collect::<Vec<_>>();

	let mut classes_select_menu = CreateSelectMenu::default();

	classes_select_menu
		.custom_id(constants::events::AUTHENTICATION_SELECT_MENU_INTERACTION)
		// TODO: translate
		.placeholder("Select a class")
		.options(|op| op.set_options(classes));

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
		Err(GoogleAuthentificationError::Timeout) => {
			let content = ctx.get("did-not-finish-auth-process", None);
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
		let content = ctx.get("error-email-not-allowed", None);
		ctx.shout(content).await?;

		return Ok(());
	}

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

	let id = match Member::with_ids(&member.user.id, &member.guild_id)
		.select(schema::members::id)
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
	};

	let new_verified_member = NewVerifiedMember {
		member_id: id,
		first_name: &user_data.first_name,
		last_name: &user_data.last_name,
		mail: &user_data.mail,
		class_id,
	};

	new_verified_member.insert().execute(&mut connection)?;

	match member.clone().add_role(ctx.discord, verified_role).await {
		Ok(_) => {}

		Err(serenity::Error::Model(serenity::ModelError::RoleNotFound)) => {
			diesel::update(schema::guilds::table.filter(schema::guilds::id.eq(member.guild_id.0)))
				.set(schema::guilds::verified_role_id.eq::<Option<u64>>(None))
				.execute(&mut connection)?;
		}

		Err(error) => return Err(error.into()),
	}

	initial_response
		.edit(|b| {
			b.content(ctx.get("authentication-successful", None))
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
fn get_and_check_login_components(
	connection: &mut DatabasePooledConnection,
	guild_id: &GuildId,
) -> Result<(RoleId, Vec<Class>, String), CheckLoginComponentsError> {
	let (verified_role, email_pattern) = {
		let (inner_role_id, email_pattern): (Option<u64>, Option<String>) =
			Guild::with_id(guild_id)
				.select((
					schema::guilds::verified_role_id,
					schema::guilds::verification_email_domain,
				))
				.first(connection)?;

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

	let classes = Class::all_from_guild(guild_id).get_results::<Class>(connection)?;

	if classes.is_empty() {
		// TODO: translate
		return Err(CheckLoginComponentsError::GuildNotTotallySetup(
			"No classes found".into(),
		));
	}

	Ok((verified_role, classes, email_pattern))
}
