//! `Diesel` models that represent database objects

use super::schema::*;
use diesel::Queryable;

/// Represent a user with `Discord` and `Google` metadata
#[derive(Queryable, Identifiable, Debug, PartialEq)]
pub struct Guild {
	/// Primary key
	pub id: u64,

	/// Guild name
	pub name: String,
	/// Guild Owner ID
	pub owner_id: u64,

	/// The id of interaction setup message
	pub setup_message_id: Option<u64>,
}

/// Use to create a new [`Guild`]
#[derive(Insertable)]
#[table_name = "guilds"]
pub struct NewGuild<'a> {
	/// Primary key
	pub id: u64,

	/// Guild name
	pub name: &'a str,
	/// Guild Owner ID
	pub owner_id: u64,

	/// The id of interaction setup message
	pub setup_message_id: Option<u64>,
}

/// Represent a user with `Discord` and `Google` metadata
#[derive(Queryable, Identifiable, Debug, PartialEq)]
#[diesel(belongs_to(Guild))]
pub struct Member {
	/// Primary key
	pub id: i32,

	/// `Discord` ID
	pub discord_id: u64,
	/// Foreign Key to [`Guild`]
	pub guild_id: u64,
	/// `Discord` username
	pub username: String,

	/// XP for messages
	pub message_xp: i32,
	/// XP for vocal
	pub vocal_xp: i32,
}

/// Use to create a new [`User`]
#[derive(Insertable)]
#[table_name = "members"]
pub struct NewMember<'a> {
	/// `Discord` ID
	pub discord_id: u64,
	/// Foreign Key to [`Guild`]
	pub guild_id: u64,
	/// `Discord` username
	pub username: &'a str,
}

/// Represent a user with `Discord` and `Google` metadata
#[derive(Queryable, Debug)]
#[diesel(belongs_to(User))]
pub struct VerifiedMember {
	/// Primary key
	pub id: i32,
	/// Foreign Key to [`User`]
	pub user_id: i32,

	/// First name
	pub first_name: String,
	/// Last name
	pub last_name: String,
	/// Account mail
	pub mail: String,
}

/// Use to create a new [`VerifiedMember`]
#[derive(Insertable)]
#[table_name = "verified_members"]
pub struct NewVerifiedMember<'a> {
	/// User ID
	pub user_id: i32,

	/// First name
	pub first_name: &'a str,
	/// Last name
	pub last_name: &'a str,
	/// Account mail
	pub mail: &'a str,
}
