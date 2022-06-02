//! `Diesel` models that represent database objects

use super::schema::*;
use crate::states::STATE;
use anyhow::{anyhow, Result};
use diesel::{ExpressionMethods, QueryDsl, Queryable, RunQueryDsl};
use oauth2::{reqwest::http_client, AccessToken, RefreshToken, TokenResponse};
use std::time::SystemTime;

/// Represent a user with `Discord` and `Google` metadata
#[derive(Queryable, Debug)]
pub struct User {
	/// Primary key
	pub id: i32,
	/// Discord ID
	pub discord_id: String,
	/// Full name
	pub full_name: String,
	/// Account mail
	pub mail: String,
	/// OAuth2 refresh token
	pub refresh_token: String,
	/// Latest OAuth2 access token
	pub access_token: String,
	/// OAuth2 access token expiration
	pub expires_at: SystemTime,
}

impl User {
	/// Get the user's access token or fetch a new one
	fn get_token(&mut self) -> Result<AccessToken> {
		if self.expires_at.elapsed()?.as_secs() > 0 {
			// TODO: refetch access token

			let res = match STATE
				.auth
				.client
				.exchange_refresh_token(&RefreshToken::new(self.refresh_token.clone()))
				.request(http_client)
			{
				Ok(res) => res,
				Err(e) => {
					log::error!("Failed to refresh token: {}", e);
					// TODO: handle this more properly
					return Err(anyhow!("Failed to refresh token"));
				}
			};

			self.access_token = res.access_token().secret().clone();
			self.expires_at = SystemTime::now() + res.expires_in().unwrap();
		}

		diesel::update(users::table.filter(users::id.eq(self.id)))
			.set((
				users::access_token.eq(&self.access_token),
				users::expires_at.eq(&self.expires_at),
			))
			.execute(&STATE.database.get()?)?;

		Ok(AccessToken::new(self.access_token.clone()))
	}
}

/// Use to create a new [`User`]
#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
	/// Discord ID
	pub discord_id: &'a str,
	/// Full name
	pub full_name: &'a String,
	/// Account mail
	pub mail: &'a str,
	/// Google OAuth2 refresh token
	pub refresh_token: &'a str,
	/// Latest OAuth2 access token
	pub access_token: &'a str,
	/// OAuth2 access token expiration
	pub expires_at: &'a SystemTime,
}
