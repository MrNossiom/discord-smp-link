table! {
	guilds (id) {
		id -> Unsigned<Bigint>,
		name -> Varchar,
		owner_id -> Unsigned<Bigint>,
		setup_message_id -> Nullable<Unsigned<Bigint>>,
	}
}

table! {
	members (id) {
		id -> Integer,
		discord_id -> Unsigned<Bigint>,
		guild_id -> Unsigned<Bigint>,
		username -> Varchar,
		message_xp -> Integer,
		vocal_xp -> Integer,
	}
}

table! {
	verified_members (id) {
		id -> Integer,
		user_id -> Integer,
		first_name -> Varchar,
		last_name -> Varchar,
		mail -> Varchar,
	}
}

joinable!(members -> guilds (guild_id));
joinable!(verified_members -> members (user_id));

allow_tables_to_appear_in_same_query!(guilds, members, verified_members,);
