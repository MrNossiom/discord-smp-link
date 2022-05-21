//! Custom loggers with their output adaptors

use log::{Level, LevelFilter, Log, Metadata, Record};
use std::{
	fs::File,
	io::Write,
	sync::Mutex,
	time::{SystemTime, UNIX_EPOCH},
};

use crate::states::STATE;

/// An adaptor that write to `stdout` and `stderr` for error levels
struct TermAdaptor {
	/// Min and Max log levels
	levels: (LevelFilter, LevelFilter),
}

impl TermAdaptor {
	/// Create a new boxed [`TermAdaptor`]
	fn boxed(levels: (Level, Level)) -> Box<Self> {
		Box::new(Self {
			levels: (levels.0.to_level_filter(), levels.1.to_level_filter()),
		})
	}
}

impl Log for TermAdaptor {
	fn enabled(&self, metadata: &Metadata) -> bool {
		self.levels.0 >= metadata.level() && self.levels.1 <= metadata.level()
	}

	fn log(&self, record: &Record) {
		print!(
			"{} - ",
			SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.expect("time went backwards")
				.as_millis()
		);

		print!("{}: ", record.level());

		print!("{} > ", record.target());

		print!("{} ", record.args());

		println!();
	}

	fn flush(&self) {
		// TODO: implement
	}
}

/// An adaptor that writes to anything that implement [`Write`]
struct WriteAdaptor<W: Write + Send + 'static> {
	/// The writer to write to
	writable: Mutex<W>,
	/// Min and Max log levels
	levels: (LevelFilter, LevelFilter),
}

impl<W: Write + Send + 'static> WriteAdaptor<W> {
	/// Create a new boxed [`WriteAdaptor`]
	fn boxed(writable: W, levels: (Level, Level)) -> Box<Self> {
		Box::new(Self {
			writable: Mutex::new(writable),
			levels: (levels.0.to_level_filter(), levels.1.to_level_filter()),
		})
	}
}

impl<W: Write + Send + 'static> Log for WriteAdaptor<W> {
	fn enabled(&self, metadata: &Metadata) -> bool {
		self.levels.0 >= metadata.level() && self.levels.1 <= metadata.level()
	}

	fn log(&self, record: &Record) {
		// TODO: change output

		let mut writable = self.writable.lock().expect("lock poisoned");

		write!(
			writable,
			"{} - ",
			SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.expect("time went backwards")
				.as_millis()
		)
		.unwrap();

		write!(writable, "{}: ", record.level()).unwrap();

		write!(writable, "{} > ", record.target()).unwrap();

		write!(writable, "{} ", record.args()).unwrap();

		writeln!(writable).unwrap();
	}

	fn flush(&self) {
		let _ = self.writable.lock().expect("lock poisoned").flush();
	}
}

/// An adaptor that sends messages to a `Discord` webhook
struct DiscordAdaptor {
	/// Min and Max log levels
	levels: (LevelFilter, LevelFilter),
}

impl DiscordAdaptor {
	/// Create a new boxed [`DiscordAdaptor`]
	fn boxed(levels: (Level, Level)) -> Box<Self> {
		Box::new(Self {
			levels: (levels.0.to_level_filter(), levels.1.to_level_filter()),
		})
	}
}

impl Log for DiscordAdaptor {
	fn enabled(&self, metadata: &Metadata) -> bool {
		self.levels.0 >= metadata.level() && self.levels.1 <= metadata.level()
	}

	fn log(&self, _record: &Record) {
		// TODO: send message to log webhook
	}

	fn flush(&self) {}
}

/// The config for the glue logger
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
	let mut adaptors: Vec<Box<dyn Log>> = vec![TermAdaptor::boxed((Level::Info, Level::Error))];

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
			(Level::Info, Level::Error),
		));
		adaptors.push(DiscordAdaptor::boxed((Level::Warn, Level::Error)));
	}

	let logger = GlueLogger::boxed(adaptors, Default::default());

	log::set_max_level(LevelFilter::Trace);
	log::set_boxed_logger(logger).expect("failed to set logger");
}
