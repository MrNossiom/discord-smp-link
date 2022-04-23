use crate::handlers::auth::AuthLink;
use anyhow::{Error, Result};
use diesel::{
	r2d2::{ConnectionManager, Pool},
	PgConnection,
};
use poise::{Command as PoiseCommand, Context as PoiseContext, Framework as PoiseFramework};

pub struct Data {
	pub database: Pool<ConnectionManager<PgConnection>>,
	pub auth: AuthLink,
}

pub type CommandError = Error;
pub type CommandResult<E = Error> = Result<(), E>;

pub type Context<'a> = PoiseContext<'a, Data, Error>;
pub type Command = PoiseCommand<Data, CommandError>;
pub type Framework = PoiseFramework<Data, CommandError>;
