//! Different log outputs adaptors and main loop

use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
	fmt::{Layer, Subscriber},
	prelude::*,
};

/// Initializes the loggers adaptors and set the global logger
#[must_use]
pub fn setup_logging() -> WorkerGuard {
	let file_appender = tracing_appender::rolling::hourly("./logs", "log");
	let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

	// TODO: add Discord logger from `gnomeutils` crate
	let global_subscriber = Subscriber::builder()
		.with_max_level(Level::INFO)
		.finish()
		.with(Layer::default().with_writer(file_writer));

	tracing::subscriber::set_global_default(global_subscriber)
		.expect("Unable to set global tracing subscriber");

	tracing::debug!("Logging setup complete");

	guard
}
