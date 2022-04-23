use oauth2::{
	basic::BasicClient, url::Url, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl,
	RevocationUrl, Scope, TokenUrl,
};
use std::env;

#[derive(Debug)]
pub struct AuthLink {
	oauth_client: BasicClient,
}

impl AuthLink {
	pub fn new() -> Self {
		let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into())
			.expect("Invalid authorization endpoint URL");
		let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".into())
			.expect("Invalid token endpoint URL");

		let (github_client_id, github_client_secret) =
			(ClientId::new("".into()), ClientSecret::new("".into()));

		let oauth_client = BasicClient::new(
			github_client_id,
			Some(github_client_secret),
			auth_url,
			Some(token_url),
		)
		.set_redirect_uri(match env::var("DEBUG").unwrap().as_str() {
			"true" => RedirectUrl::new("http://localhost:8080".into()).unwrap(),
			_ => RedirectUrl::new("http://somedumbdomain.lol".into()).unwrap(),
		})
		.set_revocation_uri(
			RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
				.expect("Invalid revocation endpoint URL"),
		);

		Self { oauth_client }
	}

	pub fn get_code(&self) -> Url {
		let (authorize_url, csrf_state) = self
			.oauth_client
			.authorize_url(CsrfToken::new_random)
			.add_scopes([
				Scope::new("https://www.googleapis.com/auth/userinfo.email".into()),
				Scope::new("https://www.googleapis.com/auth/userinfo.profile".into()),
				Scope::new("https://www.googleapis.com/auth/classroom.courses.readonly".into()),
			])
			.url();

		authorize_url
	}
}
