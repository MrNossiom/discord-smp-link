//! Small bits of `Diesel` queries to reuse across the project

use crate::database::{
	models::{Class, Guild, Member, NewMember, NewVerifiedMember},
	schema::{classes, guilds, members, verified_members},
};
use diesel::{
	helper_types::{Eq, Filter},
	prelude::*,
	query_builder::InsertStatement,
};
use poise::serenity_prelude::{GuildId, UserId};

impl Class {
	/// Select classes from their [`GuildId`]
	pub(crate) fn all_from_guild(
		guild_id: &GuildId,
	) -> Filter<classes::table, Eq<classes::guild_id, u64>> {
		classes::table.filter(classes::guild_id.eq(guild_id.0))
	}
}

impl Guild {
	/// Retrieves a guild from its ID
	pub(crate) fn with_id(guild_id: &GuildId) -> Filter<guilds::table, Eq<guilds::id, u64>> {
		guilds::table.filter(guilds::id.eq(guild_id.0))
	}
}

impl Member {
	/// Select member from his [`GuildId`] and [`UserId`]
	pub(crate) fn with_ids(
		user_id: &UserId,
		guild_id: &GuildId,
	) -> Filter<Filter<members::table, Eq<members::discord_id, u64>>, Eq<members::guild_id, u64>> {
		members::table
			.filter(members::discord_id.eq(user_id.0))
			.filter(members::guild_id.eq(guild_id.0))
	}
}

impl<'a> NewMember<'a> {
	/// Prepare a [`NewMember`] insert
	pub(crate) fn insert(
		&'a self,
	) -> InsertStatement<members::table, <&'a NewMember<'a> as Insertable<members::table>>::Values>
	{
		diesel::insert_into(members::table).values(self)
	}
}

impl<'a> NewVerifiedMember<'a> {
	/// Prepare a [`NewVerifiedMember`] insert
	pub(crate) fn insert(
		&'a self,
	) -> InsertStatement<
		verified_members::table,
		<&'a NewVerifiedMember<'a> as Insertable<verified_members::table>>::Values,
	> {
		diesel::insert_into(verified_members::table).values(self)
	}
}
