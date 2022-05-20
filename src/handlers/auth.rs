//! Handle `OAuth2` flow with users

use crate::states::Config;
use futures::Future;
use oauth2::{
	basic::{BasicClient, BasicTokenType},
	url::Url,
	AuthUrl, CsrfToken, EmptyExtraTokenFields, RedirectUrl, RevocationUrl, Scope,
	StandardTokenResponse, TokenUrl,
};
use std::{
	collections::HashMap,
	pin::Pin,
	sync::{Arc, RwLock},
	task::{Context, Poll},
	time::{Duration, Instant},
};

/// The type of the `OAuth2` response
pub type BasicTokenResponse = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

/// The type of the auth queue
pub type AuthQueue = HashMap<String, Option<BasicTokenResponse>>;

/// A manager to get redirect urls and tokens
pub struct AuthLink {
	/// The inner client used to manage the flow
	pub client: BasicClient,
	/// A queue to wait for the user to finish the flow
	pub queue: Arc<RwLock<AuthQueue>>,
}

impl AuthLink {
	/// Create a new auth link
	#[must_use]
	pub fn new(config: &Config) -> Self {
		let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into())
			.expect("invalid auth url");
		let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".into())
			.expect("invalid token url");

		let oauth_client = BasicClient::new(
			config.google_client.0.clone(),
			Some(config.google_client.1.clone()),
			auth_url,
			Some(token_url),
		)
		.set_redirect_uri(if config.production {
			RedirectUrl::new("http://somedumbdomain.lol/oauth2".into())
				.expect("invalid redirect url")
		} else {
			RedirectUrl::new(format!("http://localhost:{}/oauth2", config.port))
				.expect("invalid redirect url")
		})
		.set_revocation_uri(
			RevocationUrl::new("https://oauth2.googleapis.com/revoke".into())
				.expect("invalid revoke url"),
		);

		Self {
			client: oauth_client,
			queue: Default::default(),
		}
	}

	/// Gets a url and a future to make to user auth
	#[must_use]
	pub fn process_oauth2(&self, max_duration: Duration) -> (Url, AuthProcess) {
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
}

/// Returned by `AuthLink` for a new authentification process
/// Implement `Future` to make code more readable
pub struct AuthProcess {
	/// Abort the future if we passed the delay
	wait_until: Instant,
	/// The OAuth2 queue to handle
	queue: Arc<RwLock<AuthQueue>>,
	/// The code to recognize the request
	csrf_state: CsrfToken,
}

impl AuthProcess {
	#[must_use]
	/// Create a new auth process
	fn new(wait: Duration, queue: Arc<RwLock<AuthQueue>>, csrf_state: CsrfToken) -> Self {
		// Queue the newly created `csrf` state
		{
			let queue = queue.clone();
			let mut map = queue.write().expect("the OAuth2 RwLock is poisoned");

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
	type Output = Option<BasicTokenResponse>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let mut queue = self.queue.write().expect("the OAuth2 RwLock is poisoned");

		if Instant::now() > self.wait_until {
			Poll::Ready(None)
		} else if queue
			.get(&self.csrf_state.secret().clone())
			.unwrap()
			.is_some()
		{
			let value = queue
				.remove(&self.csrf_state.secret().clone())
				.unwrap()
				.unwrap();

			Poll::Ready(Some(value))
		} else {
			cx.waker().clone().wake();

			Poll::Pending
		}
	}
}
