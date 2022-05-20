//! The structs that represent data in the database

use std::time::SystemTime;

use super::schema::*;

/// Represent a SMP user
/// Must match the SQL in create users migration
#[allow(dead_code)]
#[derive(Queryable, Debug)]
pub struct User {
	/// Primary key
	pub id: i32,
	/// User's discord id
	pub discord_id: String,

	pub fullname: String,
	/// User's google mail
	pub mail: String,
	/// User's google OAuth2 refresh token
	pub refresh_token: String,
	pub access_token: String,
	pub expires_at: SystemTime,
}

/// Use to create a new `User`
#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
	/// User's discord id
	pub discord_id: &'a String,
	pub full_name: &'a String,
	/// User's google mail
	pub mail: &'a str,
	/// User's google OAuth2 refresh token
	pub refresh_token: &'a str,

	pub access_token: &'a str,

	pub expires_at: &'a SystemTime,
}
