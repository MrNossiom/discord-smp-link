//! Different log outputs adaptors and main loop

// pub mod discord;
pub mod term_write;

use self::term_write::{TermAdaptor, WriteAdaptor};
use crate::states::STATE;
use chrono::Utc;
use console::{style, StyledObject};
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::{fs::File, time::SystemTime};

/// The config for the [`GlueLogger`]
struct GlueLoggerConfig {
	/// If defined only logs from the given crate will be logged
	filter_crate_name: Option<&'static str>,
}

impl Default for GlueLoggerConfig {
	fn default() -> Self {
		let crate_name = env!("CARGO_PKG_NAME");

		Self {
			filter_crate_name: Some(crate_name),
		}
	}
}

/// A bridge that filter logs and transmit to multiple output adaptors
/// Does not logs anything by it self
struct GlueLogger {
	/// The multiple output adaptors
	adaptors: Vec<Box<dyn Log>>,
	/// The glue logger config
	config: GlueLoggerConfig,
}

impl GlueLogger {
	/// Create a new [`GlueLogger`] with the given adaptors
	///
	/// ```rust
	/// use crate::logging::{GlueLogger, TermAdaptor, WriteAdaptor};
	///
	/// GlueLogger::boxed(vec![
	///     TermAdaptor::boxed((Level::Info, Level::Error)),
	///     WriteAdaptor::boxed(File::create("log.txt").unwrap(), (Level::Debug, Level::Error)),
	/// ]);
	///
	fn boxed(adaptors: Vec<Box<dyn Log>>, config: GlueLoggerConfig) -> Box<Self> {
		if adaptors.is_empty() {
			panic!("no adaptors given");
		}

		Box::new(Self { adaptors, config })
	}
}

impl Log for GlueLogger {
	fn enabled(&self, meta: &Metadata) -> bool {
		self.adaptors.iter().any(|adaptor| adaptor.enabled(meta))
	}

	fn log(&self, record: &Record) {
		if let Some(crate_name) = self.config.filter_crate_name {
			if let Some(path) = record.module_path() {
				if !path.starts_with(crate_name) {
					return;
				}
			}
		}

		for adaptor in &self.adaptors {
			adaptor.log(record);
		}
	}

	fn flush(&self) {
		for adaptor in &self.adaptors {
			adaptor.flush();
		}
	}
}

/// Initializes the loggers adaptors and set the global logger
pub fn setup_logging() {
	let mut adaptors: Vec<Box<dyn Log>> = vec![TermAdaptor::boxed(Level::Info)];

	if STATE.config.production {
		adaptors.push(WriteAdaptor::boxed(
			File::create(format!(
				"logs/{}.log",
				SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.expect("time went backwards")
					.as_millis()
			))
			.expect("failed to create log file"),
			Level::Info,
		));
	}

	let logger = GlueLogger::boxed(adaptors, Default::default());

	log::set_max_level(LevelFilter::Trace);
	log::set_boxed_logger(logger).expect("failed to set logger");
}

/// Format a record in a line
trait ToPrettyRecord {
	/// Format into blank string
	fn to_pretty_record(&self) -> String;
	/// Format into color string
	fn to_pretty_record_color(&self) -> String;
}

impl<'a> ToPrettyRecord for Record<'a> {
	fn to_pretty_record(&self) -> String {
		format!(
			"[{} {} {}] {}",
			Utc::now().format("%d/%m %H:%M:%S.%3f"),
			self.level(),
			self.target()
				.strip_prefix("discord_smp_link::")
				.unwrap_or_else(|| self.target()),
			self.args()
		)
	}

	fn to_pretty_record_color(&self) -> String {
		format!(
			"[{} {} {}] {}",
			Utc::now().format("%d/%m %H:%M:%S.%3f"),
			self.level().to_styled_object(),
			style(
				self.target()
					.strip_prefix("discord_smp_link::")
					.unwrap_or_else(|| self.target())
			)
			.green(),
			self.args()
		)
	}
}

/// Convert a struct to a custom styled text
trait ToStyledObject {
	/// Convert self to a custom styled text
	fn to_styled_object(&self) -> StyledObject<&'static str>;
}

impl ToStyledObject for Level {
	fn to_styled_object(&self) -> StyledObject<&'static str> {
		let level = style(self.as_str()).bold();

		match self {
			Self::Error => level.red(),
			Self::Warn => level.yellow(),
			Self::Info => level.blue(),
			Self::Debug => level.on_cyan(),
			Self::Trace => level.on_blue(),
		}
	}
}
