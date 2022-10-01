//! Constants used in the library.

use std::time::Duration;

/// The timeout for the authentication process
pub(crate) const AUTHENTICATION_TIMEOUT: Duration = Duration::from_secs(60 * 5);

/// The interaction identifiers for buttons interactions
pub(crate) mod events {
	/// The setup message button login interaction
	pub(crate) const LOGIN_BUTTON_INTERACTION: &str = "events.setup.button.login";
	/// The setup message button logout interaction
	pub(crate) const LOGOUT_BUTTON_INTERACTION: &str = "events.setup.button.logout";

	/// The login event follow up class selection interaction
	pub(crate) const AUTHENTICATION_SELECT_MENU_INTERACTION: &str =
		"events.login.button.class-menu";
}

/// A set of URLs used in the library
pub(crate) mod urls {
	/// The Google `OAuth2` authorization endpoint
	pub(crate) const GOOGLE_AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
	/// The Google `OAuth2` token exchange endpoint
	pub(crate) const GOOGLE_TOKEN_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v3/token";
	/// The Google `OAuth2` revoke endpoint
	pub(crate) const GOOGLE_REVOKE_ENDPOINT: &str = "https://oauth2.googleapis.com/revoke";

	/// The Google `People API` endpoint
	pub(crate) const GOOGLE_PEOPLE_API_ENDPOINT: &str =
		"https://people.googleapis.com/v1/people/me";
}

/// Google `OAuth2` scopes used in the authentification process
pub(crate) mod scopes {
	/// User's email scope
	pub(crate) const USER_INFO_EMAIL: &str = "https://www.googleapis.com/auth/userinfo.email";
	/// User's informations scope
	pub(crate) const USER_INFO_PROFILE: &str = "https://www.googleapis.com/auth/userinfo.profile";

	// /// User's classrooms readonly scope
	// pub(crate) const CLASSROOM_COURSES_READONLY: &str =
	// 	"https://www.googleapis.com/auth/classroom.courses.readonly";
}
