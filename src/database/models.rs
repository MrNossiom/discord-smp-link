//! `Diesel` models that represent database objects

use super::schema::*;
use diesel::{Associations, Identifiable, Insertable, Queryable};

/// Represent a `Discord` guild
#[derive(Queryable, Identifiable, Debug, PartialEq, Eq)]
pub(crate) struct Guild {
	/// Primary key
	pub(crate) id: u64,

	/// Guild name
	pub(crate) name: String,
	/// Guild Owner ID
	pub(crate) owner_id: u64,

	/// The id of interaction setup message
	pub(crate) setup_message_id: Option<u64>,
}

/// Use to create a new [`Guild`]
#[derive(Insertable)]
#[diesel(table_name = guilds)]
pub(crate) struct NewGuild<'a> {
	/// Primary key
	pub(crate) id: u64,

	/// Guild name
	pub(crate) name: &'a str,
	/// Guild Owner ID
	pub(crate) owner_id: u64,

	/// The id of interaction setup message
	pub(crate) setup_message_id: Option<u64>,
}

/// Represent a known user with `Discord` metadata and some other informations
#[derive(Queryable, Identifiable, Associations, Debug, PartialEq, Eq)]
#[diesel(table_name = members, belongs_to(Guild))]
pub(crate) struct Member {
	/// Primary key
	pub(crate) id: i32,

	/// `Discord` ID
	pub(crate) discord_id: u64,
	/// Foreign Key to [`Guild`]
	pub(crate) guild_id: u64,
	/// `Discord` username
	pub(crate) username: String,

	/// XP for messages
	pub(crate) message_xp: i32,
	/// XP for vocal
	pub(crate) vocal_xp: i32,
}

/// Use to create a new [`Member`]
#[derive(Insertable)]
#[diesel(table_name = members)]
pub(crate) struct NewMember<'a> {
	/// `Discord` ID
	pub(crate) discord_id: u64,
	/// Foreign Key to [`Guild`]
	pub(crate) guild_id: u64,
	/// `Discord` username
	pub(crate) username: &'a str,
}

/// Represent a registered user with `Google` metadata
#[derive(Queryable, Identifiable, Debug, Associations)]
#[diesel(table_name = verified_members, belongs_to(Member))]
pub(crate) struct VerifiedMember {
	/// Primary key
	pub(crate) id: i32,
	/// Foreign Key to [`Member`]
	pub(crate) member_id: i32,

	/// First name
	pub(crate) first_name: String,
	/// Last name
	pub(crate) last_name: String,
	/// Account mail
	pub(crate) mail: String,
}

/// Use to create a new [`VerifiedMember`]
#[derive(Insertable)]
#[diesel(table_name = verified_members)]
pub(crate) struct NewVerifiedMember<'a> {
	/// User ID
	pub(crate) member_id: i32,

	/// First name
	pub(crate) first_name: &'a str,
	/// Last name
	pub(crate) last_name: &'a str,
	/// Account mail
	pub(crate) mail: &'a str,
}
