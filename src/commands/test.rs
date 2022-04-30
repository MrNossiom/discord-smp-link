use crate::{
	database::{models::SMPUser, schema::users::dsl::*},
	states::CommandResult,
	Context,
};
use poise::command;
use tokio_diesel::AsyncRunQueryDsl;

/// Connecte ton compte google SMP avec ton compte Discord pour vérifier ton identité.
#[command(prefix_command, owners_only)]
pub async fn db(ctx: Context<'_>) -> CommandResult {
	let results = users
		.load_async::<SMPUser>(&ctx.data().database)
		.await
		.expect("Error loading posts");

	println!("Displaying {} posts", results.len());

	for post in results {
		dbg!(post);
	}

	Ok(())
}
