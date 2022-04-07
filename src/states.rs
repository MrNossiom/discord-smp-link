use diesel::{
	r2d2::{ConnectionManager, Pool},
	PgConnection,
};
use poise::{serenity_prelude::*, Context as PoiseContext};
use crate::handlers::auth::AuthLink;

pub type Context<'a> = PoiseContext<'a, Data, Error>;

pub struct Data {
	pub database: Pool<ConnectionManager<PgConnection>>,
	pub auth: AuthLink,
}
