//! HTTPS and HTTP servers to answer `OAuth2` Google redirects and serve the basic web pages.
//!
//! Most of the code here comes from the [`hyper-rustls`](https://docs.rs/hyper-rustls) repository.

use super::{
	handler::{fallback, handle_error_static, handle_oauth2, TemplateResponse},
	templates::{
		ContactTemplate, IndexTemplate, PrivacyPolicyTemplate, TermsAndConditionsTemplate,
	},
};
use crate::states::{ArcData, Certificates};
use axum::{
	handler::Handler,
	routing::{self, get_service},
	Extension, Router,
};
use axum_extra::routing::SpaRouter;
use core::task::{Context, Poll};
use futures::{ready, Future};
use hyper::{
	server::{
		accept::Accept,
		conn::{AddrIncoming, AddrStream},
	},
	Body, Server,
};
use std::{io, net::SocketAddr, pin::Pin, sync::Arc};
use tokio::{
	io::{AsyncRead, AsyncWrite, ReadBuf},
	task::{self, JoinHandle},
};
use tokio_rustls::rustls::ServerConfig;
use tower::ServiceBuilder;
use tower_http::{
	services::{Redirect, ServeFile},
	trace::TraceLayer,
};

/// A task handle to a server thread
type ServerThread = JoinHandle<Result<(), hyper::Error>>;

/// Start the HTTP and HTTPS server in a new tokio task
pub(crate) fn start_server(data: ArcData) -> anyhow::Result<(ServerThread, ServerThread)> {
	// Listen on external interfaces `0.0.0.0`
	let addr_https = SocketAddr::from(([0, 0, 0, 0], data.config.port_https));
	let addr_http = SocketAddr::from(([0, 0, 0, 0], data.config.port_http));

	// TODO: find a better way to log requests
	let middleware = ServiceBuilder::new().layer(TraceLayer::new_for_http());

	let router = Router::<Body>::new()
		.route(
			"/",
			routing::get(|| async { TemplateResponse(IndexTemplate {}) }),
		)
		.route(
			"/contact",
			routing::get(|| async { TemplateResponse(ContactTemplate {}) }),
		)
		.route(
			"/privacy-policy",
			routing::get(|| async { TemplateResponse(PrivacyPolicyTemplate {}) }),
		)
		.route(
			"/terms-and-conditions",
			routing::get(|| async { TemplateResponse(TermsAndConditionsTemplate {}) }),
		)
		.route("/oauth2", routing::get(handle_oauth2))
		.route(
			"/discord",
			Redirect::temporary(data.config.discord_invite_link.clone()),
		)
		.route(
			"/favicon.ico",
			get_service(ServeFile::new("public/favicon.ico")).handle_error(handle_error_static),
		)
		.merge(SpaRouter::new("/static", "public/"))
		.fallback(fallback.into_service())
		.layer(Extension(Arc::clone(&data)))
		.layer(middleware.into_inner())
		.into_make_service();

	let tls_cfg = {
		let Certificates(certs, key) = data.certificates.clone();

		let mut cfg = rustls::ServerConfig::builder()
			.with_safe_defaults()
			.with_no_client_auth()
			.with_single_cert(certs, key)
			.map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

		// Configure ALPN to accept HTTP/2, HTTP/1.1 in that order.
		cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

		Arc::new(cfg)
	};

	// Create a TCP listener for HTTPS via tokio
	let incoming_https = AddrIncoming::bind(&addr_https)?;
	let server_https =
		Server::builder(TlsAcceptor::new(tls_cfg, incoming_https)).serve(router.clone());
	let handle_https = task::Builder::new()
		.name("Server HTTPS")
		.spawn(async move {
			tracing::debug!("Spawning HTTPS server on {}", addr_https);

			server_https.await
		})?;

	// Create a TCP listener for HTTP via tokio
	let incoming_http = AddrIncoming::bind(&addr_http)?;
	let server_http = Server::builder(incoming_http).serve(router);
	let handle_http = task::Builder::new().name("Server HTTP").spawn(async move {
		tracing::debug!("Spawning HTTP server on {}", addr_http);

		server_http.await
	})?;

	Ok((handle_https, handle_http))
}

/// The current state of an incoming connection.
enum State {
	/// The handshake is in progress.
	Handshaking(tokio_rustls::Accept<AddrStream>),
	/// The handshake has completed and streaming is in progress.
	Streaming(tokio_rustls::server::TlsStream<AddrStream>),
}

/// Represents a TLS incoming connection that is in the process of being accepted.
///
/// [`tokio_rustls::server::TlsStream`] doesn't expose constructor methods,
/// so we have to [`tokio_rustls::TlsAcceptor::accept`] and handshake to have access to it.
/// [`TlsStream`] implements `AsyncRead`/`AsyncWrite` handshaking [`tokio_rustls::Accept`] first
pub(super) struct TlsStream {
	/// The current state of the connection.
	state: State,
}

impl TlsStream {
	/// Create a new `TlsStream` from a [`ServerConfig`]
	fn new(stream: AddrStream, config: Arc<ServerConfig>) -> Self {
		let accept = tokio_rustls::TlsAcceptor::from(config).accept(stream);
		Self {
			state: State::Handshaking(accept),
		}
	}
}

impl AsyncRead for TlsStream {
	fn poll_read(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut ReadBuf,
	) -> Poll<io::Result<()>> {
		let pin = self.get_mut();
		match pin.state {
			State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
				Ok(mut stream) => {
					let result = Pin::new(&mut stream).poll_read(cx, buf);
					pin.state = State::Streaming(stream);
					result
				}
				Err(err) => Poll::Ready(Err(err)),
			},
			State::Streaming(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
		}
	}
}

impl AsyncWrite for TlsStream {
	fn poll_write(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &[u8],
	) -> Poll<io::Result<usize>> {
		let pin = self.get_mut();
		match pin.state {
			State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
				Ok(mut stream) => {
					let result = Pin::new(&mut stream).poll_write(cx, buf);
					pin.state = State::Streaming(stream);
					result
				}
				Err(err) => Poll::Ready(Err(err)),
			},
			State::Streaming(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
		}
	}

	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		match self.state {
			State::Handshaking(_) => Poll::Ready(Ok(())),
			State::Streaming(ref mut stream) => Pin::new(stream).poll_flush(cx),
		}
	}

	fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		match self.state {
			State::Handshaking(_) => Poll::Ready(Ok(())),
			State::Streaming(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
		}
	}
}

/// The struct that takes care of the TLS handshake
pub(super) struct TlsAcceptor {
	/// The [`ServerConfig`] to use for TLS handshakes.
	config: Arc<ServerConfig>,
	/// The address to listen on.
	incoming: AddrIncoming,
}

impl TlsAcceptor {
	/// Create a new [`TlsAcceptor`] from a [`ServerConfig`].
	pub(super) fn new(config: Arc<ServerConfig>, incoming: AddrIncoming) -> Self {
		Self { config, incoming }
	}
}

impl Accept for TlsAcceptor {
	type Conn = TlsStream;
	type Error = io::Error;

	fn poll_accept(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
		let pin = self.get_mut();
		match ready!(Pin::new(&mut pin.incoming).poll_accept(cx)) {
			Some(Ok(sock)) => Poll::Ready(Some(Ok(TlsStream::new(sock, pin.config.clone())))),
			Some(Err(e)) => Poll::Ready(Some(Err(e))),
			None => Poll::Ready(None),
		}
	}
}
