//! All the structs related to diesel ORM to communicate with the database

use diesel::{
	r2d2::{ConnectionManager, Pool},
	PgConnection,
};

pub mod models;
pub mod triggers;

/// The automatically generated schema by diesel.rs
pub mod schema;

pub type DatabasePool = Pool<ConnectionManager<PgConnection>>;
