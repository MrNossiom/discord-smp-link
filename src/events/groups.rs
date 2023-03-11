//! Command to disconnect Discord and Google accounts together.

use crate::{
	database::{
		models::{Group, GroupOfVerifiedMember, NewGroupOfVerifiedMember, VerifiedMember},
		prelude::*,
		schema,
	},
	states::{InteractionResult, MessageComponentContext},
	translation::Translate,
};
use anyhow::{anyhow, Context};

/// Starts the dissociate accounts process
/// Function used in the login and the setup command
#[tracing::instrument(skip(ctx), fields(caller_id = %ctx.interaction.user.id))]
pub(crate) async fn groups(
	ctx: MessageComponentContext<'_>,
	values: &Vec<String>,
) -> InteractionResult {
	let mut connection = ctx.data.database.get().await?;
	let member = ctx.guild_only_member();

	if values.len() != 1 {
		return Err(anyhow!("What the hell!").into());
	}

	let Some(member_id) = VerifiedMember::with_ids(member.user.id, member.guild_id)
		.select(schema::verified_members::member_id)
		.first(&mut connection)
		.await
		.optional()? else {
			ctx.shout("Member does not exist".to_string()).await?;

			return Ok(());
		};

	let group_id = values[0]
		.parse::<i32>()
		.context("could not parse group id")?;
	let Option::<Group>::Some(group) = Group::with_id(group_id).first(&mut connection).await.optional()? else {
		return Err(anyhow!("Group was in select menu even though it does not exist").into());

	};

	if let Option::<GroupOfVerifiedMember>::Some(group_of_verified_member) =
		GroupOfVerifiedMember::with_ids(member_id, group.id)
			.first(&mut connection)
			.await
			.optional()?
	{
		diesel::delete(&group_of_verified_member)
			.execute(&mut connection)
			.await?;

		let translate = ctx.translate("delete", None);
		ctx.shout(translate).await?;
	} else {
		let new_group_of_verified_member = NewGroupOfVerifiedMember {
			verified_member_id: member_id,
			group_id: group.id,
		};

		new_group_of_verified_member
			.insert()
			.execute(&mut connection)
			.await?;

		let translate = ctx.translate("insert", None);
		ctx.shout(translate).await?;
	}

	Ok(())
}
