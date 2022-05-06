//! Commands to debug or test a certain functionality

use crate::{
	database::{models::SMPUser, schema::users::dsl::*},
	states::CommandResult,
	Context,
};
use poise::command;
use tokio_diesel::AsyncRunQueryDsl;

#[allow(clippy::missing_docs_in_private_items)]
#[command(prefix_command, owners_only)]
pub async fn db(ctx: Context<'_>) -> CommandResult {
	let results = users
		.load_async::<SMPUser>(&ctx.data().database)
		.await
		.unwrap();

	println!("Displaying {} users", results.len());

	for post in results {
		dbg!(post);
	}

	Ok(())
}
