//! HTTPS and HTTP servers to answer `OAuth2` Google redirects and serve the basic web pages.
//!
//! Most of the code here comes from the [`oauth2`](https://docs.rs/oauth2) crate.

use super::handler::MakeRequestHandler;
use crate::states::{ArcData, Certificates};
use core::task::{Context, Poll};
use futures::{ready, Future};
use hyper::{
	server::{
		accept::Accept,
		conn::{AddrIncoming, AddrStream},
	},
	Server,
};
use hyper_staticfile::Static;
use std::{io, net::SocketAddr, path::Path, pin::Pin, sync::Arc};
use tokio::{
	io::{AsyncRead, AsyncWrite, ReadBuf},
	spawn,
	task::JoinHandle,
};
use tokio_rustls::rustls::ServerConfig;

/// A task handle to a server thread
type ServerThread = JoinHandle<Result<(), hyper::Error>>;

/// Start the HTTP and HTTPS server in a new tokio task
pub(crate) fn start_server(data: ArcData) -> anyhow::Result<(ServerThread, ServerThread)> {
	// Listen on external interfaces `0.0.0.0`
	// TODO: add different ports for http and https
	let addr_https = SocketAddr::from(([0, 0, 0, 0], data.config.port_https));
	let addr_http = SocketAddr::from(([0, 0, 0, 0], data.config.port_http));

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

	// TODO: change the path to a constant or a config value
	let static_router = Static::new(Path::new("public/"));
	// Crate the make service to handle the requests
	let service = MakeRequestHandler {
		data,
		static_router,
	};

	// Create a TCP listener for HTTPS via tokio
	let incoming = AddrIncoming::bind(&addr_https)?;
	let server_https = Server::builder(TlsAcceptor::new(tls_cfg, incoming)).serve(service.clone());
	let handle_https = spawn(async move {
		tracing::debug!("Spawning server");

		server_https.await
	});

	// Create a TCP listener for HTTP via tokio
	let server_http = Server::bind(&addr_http).serve(service);
	let handle_http = spawn(async move {
		tracing::debug!("Spawning server");

		server_http.await
	});

	Ok((handle_https, handle_http))
}

/// The current state of an incoming connection.
enum State {
	/// The handshake is in progress.
	Handshaking(tokio_rustls::Accept<AddrStream>),
	/// The handshake has completed and streaming is in progress.
	Streaming(tokio_rustls::server::TlsStream<AddrStream>),
}

// tokio_rustls::server::TlsStream doesn't expose constructor methods,
// so we have to TlsAcceptor::accept and handshake to have access to it
// TlsStream implements AsyncRead/AsyncWrite handshaking tokio_rustls::Accept first
/// Represents a TLS incoming connection that is in the process of being accepted.
pub(crate) struct TlsStream {
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
pub(crate) struct TlsAcceptor {
	/// The [`ServerConfig`] to use for TLS handshakes.
	config: Arc<ServerConfig>,
	/// The address to listen on.
	incoming: AddrIncoming,
}

impl TlsAcceptor {
	/// Create a new [`TlsAcceptor`] from a [`ServerConfig`].
	pub(crate) fn new(config: Arc<ServerConfig>, incoming: AddrIncoming) -> Self {
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
