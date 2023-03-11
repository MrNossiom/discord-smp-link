#![allow(clippy::missing_docs_in_private_items)]

//! `Diesel` models that represent database objects
// TODO: build a macro to reduce boilerplate and generate ids struct for each table with a `AsExpression` implementation

use super::schema::{
	classes, groups, groups_of_verified_members, guilds, levels, members, verified_members,
};
use diesel::{AsChangeset, Associations, Identifiable, Insertable, Queryable, Selectable};

/// Represent a `Discord` guild
#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable)]
pub(crate) struct Guild {
	pub(crate) id: u64,

	pub(crate) name: String,
	pub(crate) owner_id: u64,

	pub(crate) verification_email_domain: Option<String>,
	pub(crate) verified_role_id: Option<u64>,

	pub(crate) login_message_id: Option<u64>,
	pub(crate) groups_message_id: Option<u64>,
}

/// Use to create a new [`Guild`]
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = guilds)]
pub(crate) struct NewGuild<'a> {
	pub(crate) id: u64,

	pub(crate) name: &'a str,
	pub(crate) owner_id: u64,

	pub(crate) verification_email_domain: Option<&'a str>,
	pub(crate) verified_role_id: Option<u64>,

	pub(crate) login_message_id: Option<u64>,
	pub(crate) groups_message_id: Option<u64>,
}

/// Represent a known user with `Discord` metadata and some other informations
#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable, Associations)]
#[diesel(table_name = members, belongs_to(Guild))]
pub(crate) struct Member {
	pub(crate) id: i32,

	pub(crate) discord_id: u64,
	pub(crate) guild_id: u64,
	pub(crate) username: String,

	pub(crate) message_xp: i32,
	pub(crate) vocal_xp: i32,
}

/// Use to create a new [`Member`]
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = members)]
pub(crate) struct NewMember<'a> {
	pub(crate) discord_id: u64,
	pub(crate) guild_id: u64,
	pub(crate) username: &'a str,
}

/// Represent a registered user with `Google` metadata
#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable, Associations)]
#[diesel(table_name = verified_members, belongs_to(Member), primary_key(member_id))]
pub(crate) struct VerifiedMember {
	pub(crate) member_id: i32,

	pub(crate) mail: String,
	pub(crate) first_name: String,
	pub(crate) last_name: String,

	pub(crate) class_id: i32,
}

/// Use to create a new [`VerifiedMember`]
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = verified_members)]
pub(crate) struct NewVerifiedMember<'a> {
	pub(crate) member_id: i32,

	pub(crate) first_name: &'a str,
	pub(crate) last_name: &'a str,
	pub(crate) mail: &'a str,

	pub(crate) class_id: i32,
}

/// Represent a Level
#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable, Associations)]
#[diesel(table_name = levels, belongs_to(Guild))]
pub(crate) struct Level {
	pub(crate) id: i32,

	pub(crate) name: String,
	pub(crate) guild_id: u64,
	pub(crate) role_id: u64,
}

/// Use to create a new [`Level`]
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = levels)]
pub(crate) struct NewLevel<'a> {
	pub(crate) name: &'a str,
	pub(crate) guild_id: u64,
	pub(crate) role_id: u64,
}

/// Represent a Class
#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable, Associations)]
#[diesel(table_name = classes, belongs_to(Guild), belongs_to(Level))]
pub(crate) struct Class {
	pub(crate) id: i32,
	pub(crate) name: String,
	pub(crate) level_id: i32,

	pub(crate) guild_id: u64,
	pub(crate) role_id: u64,
}

/// Use to create a new [`Class`]
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = classes)]
pub(crate) struct NewClass<'a> {
	pub(crate) name: &'a str,
	pub(crate) level_id: i32,

	pub(crate) guild_id: u64,
	pub(crate) role_id: u64,
}

/// Represent a Group
#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable, Associations)]
#[diesel(table_name = groups, belongs_to(Guild))]
pub(crate) struct Group {
	pub(crate) id: i32,
	pub(crate) name: String,
	pub(crate) emoji: Option<String>,

	pub(crate) guild_id: u64,
	pub(crate) role_id: u64,
}

/// Use to create a new [`Group`]
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = groups)]
pub(crate) struct NewGroup<'a> {
	pub(crate) name: &'a str,
	pub(crate) emoji: Option<&'a str>,
	pub(crate) guild_id: u64,
	pub(crate) role_id: u64,
}

/// Represent a relation between a [`Group`] and a [`VerifiedMember`]
#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable, Associations)]
#[diesel(table_name = groups_of_verified_members, belongs_to(Group), belongs_to(VerifiedMember), primary_key(verified_member_id, group_id))]
pub(crate) struct GroupOfVerifiedMember {
	pub(crate) verified_member_id: i32,
	pub(crate) group_id: i32,
}

/// Use to create a new [`GroupOfVerifiedMember`]
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = groups_of_verified_members)]
pub(crate) struct NewGroupOfVerifiedMember {
	pub(crate) verified_member_id: i32,
	pub(crate) group_id: i32,
}
