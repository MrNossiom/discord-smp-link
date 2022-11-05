//! `OAuth2` flow with users

use crate::constants::{self, scopes, urls};
use crate::states::Config;
use anyhow::Context as _;
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
use std::collections::HashSet;
use std::{
	collections::HashMap,
	pin::Pin,
	sync::{Arc, RwLock},
	task::{Context, Poll},
};
use thiserror::Error;
use tokio::time::{self, Duration, Instant, Sleep};

/// The type of the `OAuth2` response
pub(crate) type BasicTokenResponse = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

/// The type of the auth queue
pub(crate) type AuthQueue = HashMap<String, BasicTokenResponse>;

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
pub(crate) struct GoogleAuthentification {
	/// The inner client used to manage the flow
	pub(crate) client: BasicClient,
	// TODO: change string to `CsrfToken` if oauth2-rs implement Eq + Hash on it
	/// A queue to wait for the user to finish the flow
	pub(crate) pending_set: Arc<RwLock<HashSet<String>>>,
	/// A queue to wait for the user to finish the flow
	pub(crate) received_queue: Arc<RwLock<AuthQueue>>,
	/// A Reqwest HTTPS client to query Google OAuth2 API
	pub(crate) http: Client,
}

impl GoogleAuthentification {
	/// Create a new [`GoogleAuthentification`]
	pub(crate) fn new(config: &Config) -> anyhow::Result<Self> {
		let auth_url = AuthUrl::new(urls::GOOGLE_AUTH_ENDPOINT.into())?;
		let token_url = TokenUrl::new(urls::GOOGLE_TOKEN_ENDPOINT.into())?;

		let redirect_url = RedirectUrl::new(format!("https://{}/oauth2", config.server_url))?;
		let revocation_url = RevocationUrl::new(urls::GOOGLE_REVOKE_ENDPOINT.into())?;

		let (client_id, client_secret) = config.google_client.clone();
		let oauth_client =
			BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
				.set_redirect_uri(redirect_url)
				.set_revocation_uri(revocation_url);

		Ok(Self {
			client: oauth_client,
			pending_set: Default::default(),
			received_queue: Default::default(),
			http: Default::default(),
		})
	}

	/// Gets a url and a future to make to user auth
	#[must_use]
	pub(crate) fn process_oauth2(&self, max_duration: Duration) -> (Url, AuthProcess) {
		let (authorize_url, csrf_state) = self
			.client
			.authorize_url(CsrfToken::new_random)
			.add_scopes([
				Scope::new(scopes::USER_INFO_EMAIL.into()),
				Scope::new(scopes::USER_INFO_PROFILE.into()),
			])
			.url();

		// Queue the newly created `csrf` state
		{
			let mut map = self.pending_set.write().expect("RwLock is poisoned");

			map.insert(csrf_state.secret().clone());
		}

		(
			authorize_url,
			AuthProcess::new(max_duration, Arc::clone(&self.received_queue), csrf_state),
		)
	}

	/// Query google for the user's email and full name
	pub(crate) async fn query_google_user_metadata(
		&self,
		token_res: &BasicTokenResponse,
	) -> Result<GoogleUserMetadata, GoogleAuthentificationError> {
		// Get this URL from a function with `fields` parameters
		let mut url =
			Url::parse(constants::urls::GOOGLE_PEOPLE_API_ENDPOINT).context("invalid query url")?;
		url.set_query(Some("personFields=names,emailAddresses"));

		let request = self
			.http
			.get(url)
			.bearer_auth(token_res.access_token().secret())
			.build()
			.context("could not build request")?;

		let response = match self.http.execute(request).await {
			Ok(response) => response,
			Err(error) => return Err(GoogleAuthentificationError::Fetch(error)),
		};

		if response.status() != StatusCode::OK {
			return Err(GoogleAuthentificationError::NonOkResponse);
		}

		let body = response
			.bytes()
			.await
			.context("could not get response bytes")?;
		let body = serde_json::from_slice::<Value>(&body)
			.map_err(GoogleAuthentificationError::MalformedResponse)?;

		let mail = body["emailAddresses"][0]["value"]
			.as_str()
			.context("failed to get email address")?
			.to_owned();

		let first_name = body["names"][0]["givenName"]
			.as_str()
			.context("failed to get first name")?
			.to_owned();
		let last_name = body["names"][0]["familyName"]
			.as_str()
			.context("failed to get last name")?
			.to_owned();

		Ok(GoogleUserMetadata {
			mail,
			first_name,
			last_name,
		})
	}
}

/// Returned by [`GoogleAuthentification`] for a new authentification process
/// Implement [`Future`] to make code more readable
#[pin_project::pin_project]
pub(crate) struct AuthProcess {
	/// Abort the future if we passed the delay
	wait_until: Instant,
	/// The OAuth2 queue to handle
	queue: Arc<RwLock<AuthQueue>>,
	/// The code to recognize the request
	csrf_state: CsrfToken,

	#[pin]
	queue_delay: Sleep,
}

impl AuthProcess {
	#[must_use]
	/// Create a new [`AuthProcess`]
	fn new(wait: Duration, queue: Arc<RwLock<AuthQueue>>, csrf_state: CsrfToken) -> Self {
		Self {
			wait_until: Instant::now() + wait,
			queue,
			csrf_state,
			queue_delay: time::sleep(Default::default()),
		}
	}
}

impl Future for AuthProcess {
	type Output = Result<BasicTokenResponse, GoogleAuthentificationError>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let mut this = self.project();

		let mut queue = this.queue.write().expect("RwLock is poisoned");

		if Instant::now() > *this.wait_until {
			return Poll::Ready(Err(GoogleAuthentificationError::Timeout));
		}

		match this.queue_delay.as_mut().poll(cx) {
			Poll::Ready(_) => {
				this.queue_delay
					.reset(Instant::now() + Duration::from_secs(5));

				match queue.remove(this.csrf_state.secret()) {
					Some(token_response) => Poll::Ready(Ok(token_response)),
					None => Poll::Pending,
				}
			}
			Poll::Pending => Poll::Pending,
		}
	}
}

/// Errors that can happen during the authentification process
#[derive(Error, Debug)]
pub(crate) enum GoogleAuthentificationError {
	/// The authentification process timed out
	#[error("The authentication timeout has expired")]
	Timeout,

	/// An error while fetching `Google`
	#[error("Could not fetch the Google API: {0}")]
	Fetch(reqwest::Error),
	/// An error while fetching `Google`
	#[error("Google answered with a non Ok status code")]
	NonOkResponse,
	/// The API response from `Google` does not contain required data
	#[error("The returned response could not be parsed")]
	MalformedResponse(serde_json::Error),

	/// Other miscellaneous errors
	#[error(transparent)]
	Other(#[from] anyhow::Error),
}
