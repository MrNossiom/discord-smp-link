//! All the pages templates derived with `askama`.

use askama::Template;

/// A template for the 404 page
#[derive(Template, Default)]
#[template(path = "404.jinja")]
pub(super) struct Code404Template {
	/// The error message of 404 page
	pub(super) ressource_path: String,
}

/// A template for the 500 page
#[derive(Template, Default)]
#[template(path = "500.jinja")]
pub(super) struct Code500Template {
	/// The error message of 500 page
	pub(super) error_message: String,
}

/// A template for the index page
#[derive(Template, Default)]
#[template(path = "index.jinja")]
pub(super) struct IndexTemplate {}

/// A template for the contact page
#[derive(Template, Default)]
#[template(path = "contact.jinja")]
pub(super) struct ContactTemplate {}

/// A template for the Privacy Policy page
#[derive(Template, Default)]
#[template(path = "privacy-policy.jinja")]
pub(super) struct PrivacyPolicyTemplate {}

/// A template for the Terms of Service page
#[derive(Template, Default)]
#[template(path = "terms-and-conditions.jinja")]
pub(super) struct TermsAndConditionsTemplate {}

/// A template for the `OAuth2` success or error page
#[derive(Template, Default)]
#[template(path = "auth.jinja")]
pub(super) struct AuthTemplate {
	/// The token response from google
	pub(super) is_success: bool,
	/// The person discord username
	pub(super) username: String,
	/// The message in case of error
	pub(super) error_message: String,
}
