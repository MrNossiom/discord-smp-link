// @generated automatically by Diesel CLI.

diesel::table! {
	classes (id) {
		id -> Integer,
		name -> Text,
		level_id -> Integer,
		guild_id -> Unsigned<Bigint>,
		role_id -> Unsigned<Bigint>,
	}
}

diesel::table! {
	groups (id) {
		id -> Integer,
		name -> Text,
		emoji -> Nullable<Char>,
		guild_id -> Unsigned<Bigint>,
		role_id -> Unsigned<Bigint>,
	}
}

diesel::table! {
	groups_of_verified_members (verified_member_id, group_id) {
		verified_member_id -> Integer,
		group_id -> Integer,
	}
}

diesel::table! {
	guilds (id) {
		id -> Unsigned<Bigint>,
		name -> Varchar,
		owner_id -> Unsigned<Bigint>,
		verification_email_domain -> Nullable<Varchar>,
		verified_role_id -> Nullable<Unsigned<Bigint>>,
		login_message_id -> Nullable<Unsigned<Bigint>>,
		groups_message_id -> Nullable<Unsigned<Bigint>>,
	}
}

diesel::table! {
	levels (id) {
		id -> Integer,
		name -> Text,
		guild_id -> Unsigned<Bigint>,
		role_id -> Unsigned<Bigint>,
	}
}

diesel::table! {
	members (id) {
		id -> Integer,
		guild_id -> Unsigned<Bigint>,
		discord_id -> Unsigned<Bigint>,
		username -> Varchar,
		message_xp -> Integer,
		vocal_xp -> Integer,
	}
}

diesel::table! {
	verified_members (member_id) {
		member_id -> Integer,
		mail -> Varchar,
		first_name -> Text,
		last_name -> Text,
		class_id -> Integer,
	}
}

diesel::joinable!(classes -> guilds (guild_id));
diesel::joinable!(classes -> levels (level_id));
diesel::joinable!(groups -> guilds (guild_id));
diesel::joinable!(groups_of_verified_members -> groups (group_id));
diesel::joinable!(groups_of_verified_members -> verified_members (verified_member_id));
diesel::joinable!(levels -> guilds (guild_id));
diesel::joinable!(members -> guilds (guild_id));
diesel::joinable!(verified_members -> classes (class_id));
diesel::joinable!(verified_members -> members (member_id));

diesel::allow_tables_to_appear_in_same_query!(
	classes,
	groups,
	groups_of_verified_members,
	guilds,
	levels,
	members,
	verified_members,
);
