//! Different log outputs adaptors and main loop

use crate::states::ArcData;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt::Layer, prelude::*, EnvFilter, Registry};

/// Initializes the loggers adaptors and set the global logger
pub(crate) fn setup_logging(data: ArcData) -> anyhow::Result<()> {
	let filter = EnvFilter::builder()
		.with_default_directive(LevelFilter::INFO.into())
		.from_env()?;

	Registry::default()
		.with(if data.config.production {
			Layer::default().json().with_filter(filter).boxed()
		} else {
			Layer::default().pretty().with_filter(filter).boxed()
		})
		.with(console_subscriber::spawn())
		.try_init()?;

	Ok(())
}
