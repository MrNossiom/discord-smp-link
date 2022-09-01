// @generated automatically by Diesel CLI.

diesel::table! {
    guilds (id) {
        id -> Unsigned<Bigint>,
        name -> Varchar,
        owner_id -> Unsigned<Bigint>,
        setup_message_id -> Nullable<Unsigned<Bigint>>,
    }
}

diesel::table! {
    members (id) {
        id -> Integer,
        discord_id -> Unsigned<Bigint>,
        guild_id -> Unsigned<Bigint>,
        username -> Varchar,
        message_xp -> Integer,
        vocal_xp -> Integer,
    }
}

diesel::table! {
    verified_members (id) {
        id -> Integer,
        member_id -> Integer,
        first_name -> Varchar,
        last_name -> Varchar,
        mail -> Varchar,
    }
}

diesel::joinable!(members -> guilds (guild_id));
diesel::joinable!(verified_members -> members (member_id));

diesel::allow_tables_to_appear_in_same_query!(
    guilds,
    members,
    verified_members,
);
