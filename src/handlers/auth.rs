use async_google_apis_common::{
	yup_oauth2::{
		authenticator_delegate::{DeviceAuthResponse, DeviceFlowDelegate},
		read_application_secret, DeviceFlowAuthenticator,
	},
	Authenticator,
};
use hyper::Client;
use hyper_rustls::HttpsConnector;
use std::{future::Future, pin::Pin, sync::Arc};

use super::apis::people::{PeopleGetParams, PeopleService};

pub struct DiscordFlowDelegate;

impl DeviceFlowDelegate for DiscordFlowDelegate {
	fn present_user_code<'a>(
		&'a self,
		device_auth_resp: &'a DeviceAuthResponse,
	) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
		println!(
			"Please enter {} at {} and grant access to this application",
			device_auth_resp.user_code, device_auth_resp.verification_uri
		);
		println!("Do not close this application until you either denied or granted access.");

		Box::pin(async {})
	}
}

pub struct AuthLink {
	pub flow: Authenticator,
	pub people_service: PeopleService,
}

impl AuthLink {
	pub async fn new() -> Self {
		let client = {
			let connection = HttpsConnector::with_native_roots();

			Client::builder().build(connection)
		};

		let flow = {
			let app_secret = read_application_secret("client_secret.json").await.unwrap();

			DeviceFlowAuthenticator::builder(app_secret)
				.flow_delegate(Box::new(DiscordFlowDelegate))
				.build()
				.await
				.unwrap()
		};

		let people_service = PeopleService::new(client.to_owned(), Arc::new(flow.to_owned()));

		people_service
			.get(&PeopleGetParams {
				resource_name: "me".into(),
				person_fields: Some("names,emailAddresses".into()),
				..Default::default()
			})
			.await
			.unwrap();

		Self {
			flow,
			people_service,
		}
	}
}
