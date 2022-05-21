//! `Diesel` models that represent database objects

use super::schema::*;
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

/// Use to create a new [`User`]
#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
	/// Discord ID
	pub discord_id: &'a String,
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
