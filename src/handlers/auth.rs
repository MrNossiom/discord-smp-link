//! `OAuth2` flow with users

use crate::states::Config;
use anyhow::{anyhow, Context as _};
use futures::Future;
use hyper::StatusCode;
use oauth2::TokenResponse;
use oauth2::{
	basic::{BasicClient, BasicTokenType},
	url::Url,
	AuthUrl, CsrfToken, EmptyExtraTokenFields, RedirectUrl, RevocationUrl, Scope,
	StandardTokenResponse, TokenUrl,
};
use reqwest::Client;
use serde_json::Value;
use std::{
	collections::HashMap,
	pin::Pin,
	sync::{Arc, RwLock},
	task::{Context, Poll},
	time::{Duration, Instant},
};
use thiserror::Error;

/// The type of the `OAuth2` response
pub(crate) type BasicTokenResponse = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

/// The type of the auth queue
pub(crate) type AuthQueue = HashMap<String, Option<BasicTokenResponse>>;

/// The information returned by google
pub(crate) struct GoogleUserMetadata {
	/// The user's mail
	pub(crate) mail: String,
	/// The user's first name
	pub(crate) first_name: String,
	/// The user's last name
	pub(crate) last_name: String,
}

/// A manager to get redirect urls and tokens
#[derive(Debug)]
pub(crate) struct AuthLink {
	/// The inner client used to manage the flow
	pub(crate) client: BasicClient,
	/// A queue to wait for the user to finish the flow
	pub(crate) queue: Arc<RwLock<AuthQueue>>,
	/// A Reqwest HTTPS client to query Google OAuth2 API
	pub(crate) http: Client,
}

impl AuthLink {
	/// Create a new [`AuthLink`]
	pub(crate) fn new(config: &Config) -> anyhow::Result<Self> {
		let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into())
			.context("invalid auth url")?;
		let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".into())
			.context("invalid token url")?;

		let oauth_client = BasicClient::new(
			config.google_client.0.clone(),
			Some(config.google_client.1.clone()),
			auth_url,
			Some(token_url),
		)
		.set_redirect_uri(
			RedirectUrl::new(format!("http://{}/oauth2", config.server_url))
				.context("invalid redirect url")?,
		)
		.set_revocation_uri(
			RevocationUrl::new("https://oauth2.googleapis.com/revoke".into())
				.context("invalid revoke url")?,
		);

		let client = Client::builder().build()?;

		Ok(Self {
			client: oauth_client,
			queue: Default::default(),
			http: client,
		})
	}

	/// Gets a url and a future to make to user auth
	#[must_use]
	pub(crate) fn process_oauth2(&self, max_duration: Duration) -> (Url, AuthProcess) {
		let (authorize_url, csrf_state) = self
			.client
			.authorize_url(CsrfToken::new_random)
			.add_scopes([
				Scope::new("https://www.googleapis.com/auth/userinfo.email".into()),
				Scope::new("https://www.googleapis.com/auth/userinfo.profile".into()),
				// Scope::new("https://www.googleapis.com/auth/classroom.courses.readonly".into()),
			])
			.url();

		(
			authorize_url,
			AuthProcess::new(max_duration, Arc::clone(&self.queue), csrf_state),
		)
	}

	/// Query google for the user's email and full name
	pub(crate) async fn query_google_user_metadata(
		&self,
		token_res: &BasicTokenResponse,
	) -> anyhow::Result<GoogleUserMetadata> {
		let request = self
			.http
			// Get this URL from a function with `fields` parameters
			.get("https://people.googleapis.com/v1/people/me?personFields=names,emailAddresses")
			.bearer_auth(token_res.access_token().secret())
			.build()?;

		let response = self
			.http
			.execute(request)
			.await
			.context("failed to query Google")?;

		if response.status() != StatusCode::OK {
			return Err(anyhow!("Google answered with a non Ok status code"));
		}

		let body = response.bytes().await?;
		let body = serde_json::from_slice::<Value>(&body)?;

		let mail = body["emailAddresses"][0]["value"]
			.as_str()
			.context("Failed to get email address")?
			.to_owned();

		let first_name = body["names"][0]["givenName"]
			.as_str()
			.context("Failed to get first name")?
			.to_owned();
		let last_name = body["names"][0]["familyName"]
			.as_str()
			.context("Failed to get last name")?
			.to_owned();

		Ok(GoogleUserMetadata {
			mail,
			first_name,
			last_name,
		})
	}
}

/// Returned by [`AuthLink`] for a new authentification process
/// Implement [`Future`] to make code more readable
pub(crate) struct AuthProcess {
	/// Abort the future if we passed the delay
	wait_until: Instant,
	/// The OAuth2 queue to handle
	queue: Arc<RwLock<AuthQueue>>,
	/// The code to recognize the request
	csrf_state: CsrfToken,
}

impl AuthProcess {
	#[must_use]
	/// Create a new [`AuthProcess`]
	fn new(wait: Duration, queue: Arc<RwLock<AuthQueue>>, csrf_state: CsrfToken) -> Self {
		// Queue the newly created `csrf` state
		{
			let queue = queue.clone();
			let mut map = queue.write().expect("RwLock is poisoned");

			map.insert(csrf_state.secret().clone(), None);
		}

		Self {
			wait_until: Instant::now() + wait,
			queue,
			csrf_state,
		}
	}
}

impl Future for AuthProcess {
	type Output = Result<BasicTokenResponse, AuthProcessError>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let mut queue = self.queue.write().expect("RwLock is poisoned");

		if Instant::now() > self.wait_until {
			return Poll::Ready(Err(AuthProcessError::Timeout));
		}

		match queue.get(&self.csrf_state.secret().clone()) {
			None => Poll::Ready(Err(AuthProcessError::NotQueued)),
			Some(Some(_)) => {
				let token = queue
					.remove(&self.csrf_state.secret().clone())
					.expect("entry was checked just before")
					.expect("entry was checked just before");

				Poll::Ready(Ok(token))
			}
			Some(None) => {
				// Add a delay to avoid spamming the queue

				cx.waker().clone().wake();

				Poll::Pending
			}
		}
	}
}

/// Errors that can happen during the authentification process
#[derive(Error, Debug)]
pub(crate) enum AuthProcessError {
	/// The authentification process timed out
	#[error("The authentication timeout has expired")]
	Timeout,
	/// The authentification process was not queued
	#[error("The given csrf state is not queued")]
	NotQueued,
}
