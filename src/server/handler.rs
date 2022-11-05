// TODO: remove when fixed in clippy code `https://github.com/rust-lang/rust-clippy/pull/9486`
// ? Necessary to avoid clippy warning for `FromForm` derives
#![allow(clippy::unnecessary_lazy_evaluations)]

//! The request handlers that serves content

use crate::states::ArcData;
use oauth2::{reqwest::async_http_client, AuthorizationCode};
use rocket::{response::Redirect, FromForm, Request, State};
use rocket_dyn_templates::{context, Template};

/// The parameters for the `OAuth2` callback endpoint
#[derive(FromForm)]
pub(crate) struct OAuth2Params {
	/// The code returned by the `OAuth2` provider
	code: String,
	/// The CSRF token to identify the request
	state: String,
}

// TODO: show more comprehensive errors to the user
/// Handle requests to `/oauth2` endpoints
#[rocket::get("/oauth2?<params..>")]
pub(super) async fn handle_oauth2(data: &State<ArcData>, params: OAuth2Params) -> Template {
	{
		let queue = data.auth.pending_set.read().expect("poisoned");

		if queue.get(&params.state).is_none() {
			return Template::render(
				"auth",
				context! { is_success: false, message: "The given 'state' wasn't queued anymore" },
			);
		};
	}

	let oauth2_response = data
		.auth
		.client
		.exchange_code(AuthorizationCode::new(params.code))
		.request_async(async_http_client)
		.await;

	let token_response = match oauth2_response {
		Ok(token_res) => token_res,
		Err(error) => {
			return Template::render(
				"auth",
				context! { is_success: false, message: error.to_string() },
			);
		}
	};

	{
		let mut queue = data.auth.received_queue.write().expect("poisoned");

		queue.insert(params.state, token_response);
	}

	Template::render("auth", context! { is_success: true, username: "" })
}

/// Serve the index page
#[rocket::get("/")]
pub(super) fn index() -> Template {
	Template::render("index", context! {})
}

/// Serve the Contact page
#[rocket::get("/contact")]
pub(super) fn contact() -> Template {
	Template::render("contact", context! {})
}

/// Serve the Privacy Policy page
#[rocket::get("/privacy-policy")]
pub(super) fn privacy_policy() -> Template {
	Template::render("privacy-policy", context! {})
}

/// Serve the Terms and Conditions page
#[rocket::get("/terms-and-conditions")]
pub(super) fn terms_and_conditions() -> Template {
	Template::render("terms-and-conditions", context! {})
}

/// Redirects to the main discord server
#[rocket::get("/discord")]
pub(super) fn discord_redirect(data: &State<ArcData>) -> Redirect {
	Redirect::to(format!(
		"https://discord.gg/{}",
		data.config.discord_invite_code
	))
}

/// Catch the `404` status code
#[rocket::catch(404)]
pub(super) fn catch_404(req: &Request<'_>) -> Template {
	Template::render(
		"404",
		context! { ressource_path: req.uri().path().to_string() },
	)
}

/// Catch the `500` status code
#[rocket::catch(500)]
pub(super) fn catch_500() -> Template {
	Template::render("500", context! { message: "Internal Server Error" })
}
