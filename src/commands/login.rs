use crate::Context;
use poise::{command, serenity_prelude::*};

#[command(slash_command)]
pub async fn login(_ctx: Context<'_>) -> Result<()> {
	Ok(())
}
