#[allow(dead_code)]
#[derive(Queryable, Debug)]
pub struct SMPUser {
	pub id: i32,
	pub discord_id: String,
	mail: Option<String>,
	refresh_token: Option<String>,
}
