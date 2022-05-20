table! {
	users (id) {
		id -> Int4,
		discord_id -> Varchar,
		full_name -> Varchar,
		mail -> Varchar,
		refresh_token -> Varchar,
		access_token -> Varchar,
		expires_at -> Timestamp,
	}
}
