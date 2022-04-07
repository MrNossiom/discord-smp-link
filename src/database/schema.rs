table! {
	users (id) {
		id -> Int4,
		username -> Varchar,
		mail -> Nullable<Varchar>,
		refresh_token -> Nullable<Text>,
	}
}
