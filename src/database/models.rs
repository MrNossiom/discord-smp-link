//! The structs that represent data in the database

/// Represent a SMP user
#[allow(dead_code)]
#[derive(Queryable, Debug)]
pub struct SMPUser {
	/// Primary key
	pub id: i32,
	/// User's discord id
	pub discord_id: String,
	/// User's mail
	mail: Option<String>,
	/// User's google oauth2 refresh token
	refresh_token: Option<String>,
}
