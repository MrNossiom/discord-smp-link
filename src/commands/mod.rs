mod login;

pub use login::*;

use crate::states::{CommandResult, Context};
use poise::{builtins::register_application_commands, command};

#[command(prefix_command, owners_only, hide_in_help)]
pub async fn register(ctx: Context<'_>, #[flag] global: bool) -> CommandResult {
	ctx.say("Registered!").await?;
	match register_application_commands(ctx, global).await {
		Ok(_) => Ok(()),
		Err(e) => {
			dbg!(&e);
			Err(e)
		}
	}
	.unwrap();

	Ok(())
}
