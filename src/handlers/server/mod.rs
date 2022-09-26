//! Servers configs and request handlers to serve `OAuth2` callbacks and the web pages.

pub(self) mod error;
pub(self) mod handler;
pub(self) mod templates;
pub(self) mod tls_server;

pub(crate) use tls_server::start_server;
