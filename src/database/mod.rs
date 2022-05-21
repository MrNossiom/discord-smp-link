//! Models and triggers related to database management

use diesel::{
	r2d2::{ConnectionManager, Pool},
	PgConnection,
};

pub mod models;
pub mod triggers;

/// The automatically generated schema by `Diesel`
pub mod schema;

/// The type alias for a Postgres connection pool
pub type DatabasePool = Pool<ConnectionManager<PgConnection>>;
