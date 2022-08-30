//! `Diesel` models that represent database objects

use super::schema::*;
use diesel::Queryable;

/// Represent a `Discord` guild
#[derive(Queryable, Identifiable, Debug, PartialEq, Eq)]
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
#[diesel(table_name = guilds)]
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

/// Represent a known user with `Discord` metadata and some other informations
#[derive(Queryable, Identifiable, Debug, PartialEq, Eq)]
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

/// Use to create a new [`Member`]
#[derive(Insertable)]
#[diesel(table_name = members)]
pub struct NewMember<'a> {
	/// `Discord` ID
	pub discord_id: u64,
	/// Foreign Key to [`Guild`]
	pub guild_id: u64,
	/// `Discord` username
	pub username: &'a str,
}

/// Represent a registered user with `Google` metadata
#[derive(Queryable, Debug)]
#[diesel(belongs_to(User))]
pub struct VerifiedMember {
	/// Primary key
	pub id: i32,
	/// Foreign Key to [`Member`]
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
#[diesel(table_name = verified_members)]
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
