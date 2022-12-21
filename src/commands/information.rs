//! Context Command for informations about a verified member.

use crate::{
	database::{models::VerifiedMember, prelude::*},
	states::{ApplicationContext, ApplicationContextPolyfill, InteractionResult},
	translation::Translate,
};
use fluent::fluent_args;
use poise::{command, serenity_prelude::User};

/// Show informations about a registered member
#[command(slash_command, context_menu_command = "Informations", guild_only)]
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user().id))]
pub(crate) async fn information(ctx: ApplicationContext<'_>, user: User) -> InteractionResult {
	let guild_id = ctx.guild_only_id();

	let verified_member: VerifiedMember = match VerifiedMember::with_ids(user.id, guild_id)
		.select(VerifiedMember::as_select())
		.first::<VerifiedMember>(&mut ctx.data.database.get().await?)
		.await
	{
		Ok(x) => x,
		Err(DieselError::NotFound) => {
			let get = ctx.translate(
				"error-member-not-verified",
				Some(&fluent_args!["user" => user.name]),
			);
			ctx.shout(get).await?;

			return Ok(());
		}
		Err(err) => return Err(err.into()),
	};

	ctx.send(|builder| {
		builder.ephemeral(true).embed(|embed| {
			embed
				.title(format!(
					"{} {}",
					verified_member.first_name, verified_member.last_name
				))
				.field("Mail", verified_member.mail, false)
				.color(0x0000_FF00)
				.footer(|footer| footer.text("Discord SMP Link © 2023"))
		})
	})
	.await?;

	Ok(())
}
