mod login;

pub use login::login;

use crate::states::Context;
use poise::{builtins::register_application_commands, serenity_prelude::*};

#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<()> {
	ctx.say("Registered!").await?;
	match register_application_commands(ctx, global).await {
		Ok(_) => {}
		Err(e) => {
			dbg!(e);
		}
	};
	println!("Registered commands.");

	Ok(())
}
