//! All the pages templates derived with `askama`.

use askama::Template;

/// A template for the 404 page
#[derive(Template, Default)]
#[template(path = "404.jinja")]
pub(crate) struct Code404Template<'a> {
	/// The error message of 404 page
	pub(crate) ressource_path: &'a str,
}

/// A template for the 500 page
#[derive(Template, Default)]
#[template(path = "500.jinja")]
pub(crate) struct Code500Template<'a> {
	/// The error message of 500 page
	pub(crate) error_message: &'a str,
}

/// A template for the index page
#[derive(Template, Default)]
#[template(path = "index.jinja")]
pub(crate) struct IndexTemplate {}

/// A template for the contact page
#[derive(Template, Default)]
#[template(path = "contact.jinja")]
pub(crate) struct ContactTemplate {}

/// A template for the Privacy Policy page
#[derive(Template, Default)]
#[template(path = "privacy-policy.jinja")]
pub(crate) struct PrivacyPolicyTemplate {}

/// A template for the Terms of Service page
#[derive(Template, Default)]
#[template(path = "terms-and-conditions.jinja")]
pub(crate) struct TermsAmdConditionsTemplate {}

/// A template for the `OAuth2` success or error page
#[derive(Template, Default)]
#[template(path = "auth.jinja")]
pub(crate) struct AuthTemplate<'a> {
	/// The token response from google
	pub(crate) is_success: bool,
	/// The person discord username
	pub(crate) username: &'a str,
	/// The message in case of error
	pub(crate) error_message: &'a str,
}
