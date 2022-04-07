mod models;
mod schema;

use diesel::{
	pg::PgConnection,
	r2d2::{ConnectionManager, Pool},
};
use std::env;

pub fn establish_connection() -> Pool<ConnectionManager<PgConnection>> {
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

	let manager = ConnectionManager::<PgConnection>::new(database_url);

	Pool::builder()
		.build(manager)
		.expect("Failed to create pool.")
}
