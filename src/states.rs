use poise::{serenity_prelude::*, Context as PoiseContext};

pub type Context<'a> = PoiseContext<'a, Data, Error>;

pub struct Data {}
