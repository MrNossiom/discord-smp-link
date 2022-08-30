//! Triggers that query servers and manipulate the database

use super::{
	models::NewVerifiedMember,
	schema::{members, verified_members},
};
use crate::{handlers::auth::BasicTokenResponse, states::Data};
use anyhow::{anyhow, Result};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use hyper::{body::to_bytes, Body, Client, Request, StatusCode};
use hyper_rustls::HttpsConnectorBuilder;
use oauth2::TokenResponse;
use poise::serenity_prelude::{Member, User};
use serde_json::Value;
use std::sync::Arc;

/// Insert a new user into the database
/// Query google for the user's email and full name
pub async fn new_verified_member(
	data: Arc<Data>,
	member: &Member,
	res: &BasicTokenResponse,
) -> Result<()> {
	let user_data = match query_google_user_metadata(res).await {
		Ok(user_data) => user_data,
		Err(e) => {
			tracing::error!("Failed to query google user metadata: {}", e);

			return Err(anyhow!("Failed to query google user metadata"));
		}
	};

	let id = members::table
		.filter(members::discord_id.eq(member.user.id.0))
		.filter(members::guild_id.eq(member.guild_id.0))
		.select(members::id)
		.first(&mut data.database.get()?)?;

	let new_verified_member = NewVerifiedMember {
		user_id: id,
		first_name: &user_data.first_name,
		last_name: &user_data.last_name,
		mail: &user_data.mail,
	};

	diesel::insert_into(verified_members::table)
		.values(new_verified_member)
		.execute(&mut data.database.get()?)?;

	Ok(())
}

/// Remove a user from the database
pub fn delete_user(data: Arc<Data>, user: &User) -> Result<()> {
	// TODO: change cast to be functional
	diesel::delete(members::table)
		.filter(members::discord_id.eq(user.id.0))
		.execute(&mut data.database.get()?)?;

	Ok(())
}

/// The information returned by google
struct GoogleUserMetadata {
	/// The user's mail
	mail: String,
	/// The user's first name
	first_name: String,
	/// The user's last name
	last_name: String,
}

// TODO: move elsewhere
/// Query google for the user's email and full name
async fn query_google_user_metadata(token_res: &BasicTokenResponse) -> Result<GoogleUserMetadata> {
	let https = HttpsConnectorBuilder::new()
		.with_native_roots()
		.https_only()
		.enable_http1()
		.build();
	let client: Client<_, Body> = Client::builder().build(https);

	let req = Request::builder()
		.header(
			"Authorization",
			format!("Bearer {}", token_res.access_token().secret()),
		)
		.uri("https://people.googleapis.com/v1/people/me?personFields=names,emailAddresses")
		.body(Body::empty())
		.expect("Failed to build request");

	match client.request(req).await {
		Ok(res) => {
			if res.status() != StatusCode::OK {
				return Err(anyhow!("Failed to query google user metadata"));
			}

			let body = to_bytes(res.into_body())
				.await
				.expect("Failed to read response");
			let body = serde_json::from_slice::<Value>(&body).expect("Failed to parse body");

			let mail = body["emailAddresses"][0]["value"]
				.as_str()
				.expect("Failed to get email address")
				.to_owned();

			let first_name = body["names"][0]["givenName"]
				.as_str()
				.expect("Failed to get first name")
				.to_owned();
			let last_name = body["names"][0]["familyName"]
				.as_str()
				.expect("Failed to get last name")
				.to_owned();

			Ok(GoogleUserMetadata {
				mail,
				first_name,
				last_name,
			})
		}
		Err(error) => {
			tracing::error!("Failed to query google: {}", error);

			Err(anyhow!("Failed to query google"))
		}
	}
}
