//! Constants used in the library.

use std::time::Duration;

/// The timeout for the authentication process
pub(crate) const AUTHENTICATION_TIMEOUT: Duration = Duration::from_secs(60 * 5);

/// The interaction identifiers for buttons interactions
pub(crate) mod events {
	/// The setup message button interaction
	pub(crate) const LOGIN_BUTTON_INTERACTION: &str = "events.login_button_interaction";
}
