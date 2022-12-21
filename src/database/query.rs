// Return types tends to be complex in here
#![allow(clippy::type_complexity)]

//! Small bits of `Diesel` queries to reuse across the project

use super::{
	models::{Level, NewLevel},
	prelude::*,
	schema::levels,
};
use crate::database::{
	models::{
		Class, Group, Guild, Member, NewClass, NewGroup, NewMember, NewVerifiedMember,
		VerifiedMember,
	},
	schema::{classes, groups, guilds, members, verified_members},
};
use diesel::{
	dsl::insert_into,
	helper_types::{Eq, Filter, Find, InnerJoin},
	query_builder::InsertStatement,
};
use poise::serenity_prelude::{GuildId, UserId};

impl Class {
	/// Select classes from their [`GuildId`]
	#[inline]
	pub(crate) fn all_from_guild(
		guild_id: GuildId,
	) -> Filter<classes::table, Eq<classes::guild_id, u64>> {
		classes::table.filter(classes::guild_id.eq(guild_id.0))
	}

	/// Select classes from their [`GuildId`]
	#[inline]
	pub(crate) fn all_from_level(
		level_id: i32,
	) -> Filter<classes::table, Eq<classes::level_id, i32>> {
		classes::table.filter(classes::level_id.eq(level_id))
	}

	/// Select class from his `id`
	#[inline]
	pub(crate) fn with_id(class_id: i32) -> Find<classes::table, i32> {
		classes::table.find(class_id)
	}
}

impl<'a> NewClass<'a> {
	/// Prepare a [`NewClass`] insert
	#[inline]
	pub(crate) fn insert(
		&'a self,
	) -> InsertStatement<classes::table, <&'a NewClass<'a> as Insertable<classes::table>>::Values> {
		insert_into(classes::table).values(self)
	}
}

impl Guild {
	/// Retrieves a guild from its ID
	#[inline]
	pub(crate) fn with_id(guild_id: GuildId) -> Find<guilds::table, u64> {
		guilds::table.find(guild_id.0)
	}
}

impl Member {
	/// Select member from his [`GuildId`] and [`UserId`]
	#[inline]
	pub(crate) fn with_ids(
		user_id: UserId,
		guild_id: GuildId,
	) -> Filter<Filter<members::table, Eq<members::discord_id, u64>>, Eq<members::guild_id, u64>> {
		members::table
			.filter(members::discord_id.eq(user_id.0))
			.filter(members::guild_id.eq(guild_id.0))
	}
}

impl<'a> NewMember<'a> {
	/// Prepare a [`NewMember`] insert
	#[inline]
	pub(crate) fn insert(
		&'a self,
	) -> InsertStatement<members::table, <&'a NewMember<'a> as Insertable<members::table>>::Values>
	{
		insert_into(members::table).values(self)
	}
}

impl<'a> NewVerifiedMember<'a> {
	/// Prepare a [`NewVerifiedMember`] insert
	#[inline]
	pub(crate) fn insert(
		&'a self,
	) -> InsertStatement<
		verified_members::table,
		<&'a NewVerifiedMember<'a> as Insertable<verified_members::table>>::Values,
	> {
		insert_into(verified_members::table).values(self)
	}
}

impl Group {
	/// Select groups from their [`GuildId`]
	#[inline]
	pub(crate) fn all_from_guild(
		guild_id: GuildId,
	) -> Filter<groups::table, Eq<groups::guild_id, u64>> {
		groups::table.filter(groups::guild_id.eq(guild_id.0))
	}

	/// Select group from his `id`
	#[inline]
	pub(crate) fn with_id(group_id: i32) -> Find<groups::table, i32> {
		groups::table.find(group_id)
	}
}

impl<'a> NewGroup<'a> {
	/// Prepare a [`NewGroup`] insert
	#[inline]
	pub(crate) fn insert(
		&'a self,
	) -> InsertStatement<groups::table, <&'a NewGroup<'a> as Insertable<groups::table>>::Values> {
		insert_into(groups::table).values(self)
	}
}

impl Level {
	/// Select groups from their [`GuildId`]
	#[inline]
	pub(crate) fn all_from_guild(
		guild_id: GuildId,
	) -> Filter<levels::table, Eq<levels::guild_id, u64>> {
		levels::table.filter(levels::guild_id.eq(guild_id.0))
	}

	/// Select level from his `id`
	#[inline]
	pub(crate) fn with_id(level_id: i32) -> Find<levels::table, i32> {
		levels::table.find(level_id)
	}
}

impl<'a> NewLevel<'a> {
	/// Prepare a [`NewLevel`] insert
	#[inline]
	pub(crate) fn insert(
		&'a self,
	) -> InsertStatement<levels::table, <&'a NewLevel<'a> as Insertable<levels::table>>::Values> {
		insert_into(levels::table).values(self)
	}
}

impl VerifiedMember {
	/// Select verified member from his [`GuildId`] and [`UserId`]
	#[inline]
	pub(crate) fn with_ids(
		user_id: UserId,
		guild_id: GuildId,
	) -> Filter<
		Filter<InnerJoin<verified_members::table, members::table>, Eq<members::discord_id, u64>>,
		Eq<members::guild_id, u64>,
	> {
		verified_members::table
			.inner_join(members::table)
			.filter(members::discord_id.eq(user_id.0))
			.filter(members::guild_id.eq(guild_id.0))
	}

	/// Filter from the foreign key of a [`Member`]
	#[inline]
	pub(crate) fn from_member_id(
		member_id: i32,
	) -> Filter<verified_members::table, Eq<verified_members::member_id, i32>> {
		verified_members::table.filter(verified_members::member_id.eq(member_id))
	}
}
