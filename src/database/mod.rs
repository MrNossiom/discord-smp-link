//! Models and triggers related to database management

use diesel::{
	mysql::Mysql,
	r2d2::{ConnectionManager, Pool},
	MysqlConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::error::Error;

pub(crate) mod models;
pub(crate) mod triggers;

/// The automatically generated schema by `Diesel`
#[rustfmt::skip]
pub(crate) mod schema;

/// The type alias for a Postgres connection pool
pub(crate) type DatabasePool = Pool<ConnectionManager<MysqlConnection>>;

/// The migrations to apply to the database
pub(crate) const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Applies the migrations to the database
pub(crate) fn run_migrations(
	connection: &mut impl MigrationHarness<Mysql>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
	connection.run_pending_migrations(MIGRATIONS)?;

	Ok(())
}
