//! Triggers that query servers and manipulate the database

use super::{models::NewUser, schema::users};
use crate::{handlers::auth::BasicTokenResponse, states::STATE};
use anyhow::{anyhow, Result};
use diesel::{ExpressionMethods, RunQueryDsl};
use hyper::{body::to_bytes, Body, Client, Request};
use hyper_rustls::HttpsConnector;
use oauth2::TokenResponse;
use poise::serenity_prelude::User;
use serde_json::Value;
use std::time::SystemTime;

/// Insert a new user into the database
/// Query google for the user's email and full name
pub async fn new_user(user: &User, res: &BasicTokenResponse) -> Result<()> {
	let user_data = match query_google_user_metadata(res).await {
		Ok(user_data) => user_data,
		Err(e) => {
			log::error!("Failed to query google user metadata: {}", e);

			return Err(anyhow!("Failed to query google user metadata"));
		}
	};

	let new_user = NewUser {
		discord_id: &user.id.to_string(),
		full_name: &user_data.full_name,
		mail: &user_data.mail,
		refresh_token: res.refresh_token().unwrap().secret().as_str(),
		access_token: res.access_token().secret().as_str(),
		expires_at: &(SystemTime::now() + res.expires_in().unwrap()),
	};

	diesel::insert_into(users::table)
		.values(&new_user)
		.on_conflict_do_nothing()
		.execute(&STATE.database.get()?)?;

	Ok(())
}

/// Remove a user from the database
pub fn delete_user(user: &User) -> Result<()> {
	diesel::delete(users::table)
		.filter(users::discord_id.eq(&user.id.to_string()))
		.execute(&STATE.database.get()?)?;

	Ok(())
}

struct GoogleUserMetadata {
	mail: String,
	full_name: String,
}

// TODO: move elsewhere
/// Query google for the user's email and full name
async fn query_google_user_metadata(tkn_res: &BasicTokenResponse) -> Result<GoogleUserMetadata> {
	let https = HttpsConnector::with_native_roots();
	let client: Client<_, Body> = Client::builder().build(https);

	let req = Request::builder()
		.header(
			"Authorization",
			format!("Bearer {}", tkn_res.access_token().secret()),
		)
		.uri("https://people.googleapis.com/v1/people/me?personFields=names,emailAddresses")
		.body(Body::empty())
		.expect("Failed to build request");

	match client.request(req).await {
		Ok(res) => {
			let body = to_bytes(res.into_body())
				.await
				.expect("Failed to read response");
			let body = serde_json::from_slice::<Value>(&body).expect("Failed to parse body");

			let mail = body["emailAddresses"][0]["value"]
				.as_str()
				.unwrap()
				.to_owned();
			let full_name = body["names"][0]["displayName"].as_str().unwrap().to_owned();

			Ok(GoogleUserMetadata { mail, full_name })
		}
		Err(error) => {
			log::error!("Failed to query google: {}", error);

			Err(anyhow!("Failed to query google"))
		}
	}
}
