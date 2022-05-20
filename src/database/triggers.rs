use super::{models::NewUser, schema::users};
use crate::{handlers::auth::BasicTokenResponse, states::STATE};
use anyhow::Result;
use diesel::RunQueryDsl;
use oauth2::TokenResponse;
use poise::serenity_prelude::User;
use std::time::SystemTime;

pub fn new_user(user: &User, res: &BasicTokenResponse) -> Result<()> {
	let (mail, full_name) = query_google_user_metadata(&res);

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

fn query_google_user_metadata(_res: &BasicTokenResponse) -> (String, String) {
	("".into(), "".into())
}
