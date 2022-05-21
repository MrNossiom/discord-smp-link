//! Triggers that query servers and manipulate the database

use super::{models::NewUser, schema::users};
use crate::{handlers::auth::BasicTokenResponse, states::STATE};
use anyhow::Result;
use diesel::{ExpressionMethods, RunQueryDsl};
use oauth2::TokenResponse;
use poise::serenity_prelude::User;
use std::time::SystemTime;

/// Insert a new user into the database
/// Query google for the user's email and full name
pub fn new_user(user: &User, res: &BasicTokenResponse) -> Result<()> {
	let (mail, full_name) = query_google_user_metadata(res);

	let new_user = NewUser {
		discord_id: &user.id.to_string(),
		full_name: &full_name,
		mail: &mail,
		refresh_token: res.refresh_token().unwrap().secret().as_str(),
		access_token: res.access_token().secret().as_str(),
		expires_at: &(SystemTime::now() + res.expires_in().unwrap()),
	};

	diesel::insert_into(users::table)
		.values(&new_user)
		.on_conflict_do_nothing()
		.execute(&STATE.database.get()?)?;

	Ok(())
}

/// Remove a user from the database
pub fn delete_user(user: &User) -> Result<()> {
	diesel::delete(users::table)
		.filter(users::discord_id.eq(&user.id.to_string()))
		.execute(&STATE.database.get()?)?;

	Ok(())
}

// TODO: move elsewhere
/// Query google for the user's email and full name
fn query_google_user_metadata(_res: &BasicTokenResponse) -> (String, String) {
	log::error!("Google calls is not implemented yet, line: {}", line!());
	("".into(), "".into())
}
