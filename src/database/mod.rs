//! Models and triggers related to database management

use diesel::{
	mysql::Mysql,
	r2d2::{ConnectionManager, Pool, PooledConnection},
	MysqlConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub(crate) mod models;
/// The automatically generated schema by `Diesel`
#[rustfmt::skip]
pub(crate) mod schema;

pub(crate) use diesel::result::Error as DieselError;

/// The type alias for a Postgres connection pool
pub(crate) type DatabasePool = Pool<ConnectionManager<MysqlConnection>>;

/// The type alias for a Postgres connection handle
pub(crate) type DatabasePooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

/// The migrations to apply to the database
pub(crate) const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Applies the migrations to the database
pub(crate) fn run_migrations(
	connection: &mut impl MigrationHarness<Mysql>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
	let migrations_applied = connection.run_pending_migrations(MIGRATIONS)?;

	tracing::debug!(migrations = ?migrations_applied, "Applied migrations");

	Ok(())
}
