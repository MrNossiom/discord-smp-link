//! The schemas to interact with the database

table! {
	users (id) {
		id -> Int4,
		discord_id -> Varchar,
		mail -> Nullable<Varchar>,
		refresh_token -> Nullable<Text>,
	}
}
