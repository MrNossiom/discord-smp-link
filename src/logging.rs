//! Custom loggers with their output adaptors

use log::{Level, LevelFilter, Log, Metadata, Record};
use std::{fs::File, io::Write, sync::Mutex, time::SystemTime};

trait LogAdaptor: Send + Sync {
	fn level(&self) -> LevelFilter;
	fn log(&self, record: &Record);
	fn flush(&self);
}

struct TermLogger {
	min_level: LevelFilter,
}

impl TermLogger {
	fn boxed(level: Level) -> Box<Self> {
		Box::new(Self {
			min_level: level.to_level_filter(),
		})
	}
}

impl LogAdaptor for TermLogger {
	fn level(&self) -> LevelFilter {
		self.min_level
	}

	fn log(&self, record: &Record) {
		// TODO: log something
		todo!("log something")
	}

	fn flush(&self) {
		// TODO: impl
		todo!("flush something")
	}
}

struct WriteLogger<W: Write + Send + 'static> {
	writable: Mutex<W>,
	min_level: LevelFilter,
}

impl<W: Write + Send + 'static> WriteLogger<W> {
	fn boxed(writable: W, level: Level) -> Box<Self> {
		Box::new(Self {
			writable: Mutex::new(writable),
			min_level: level.to_level_filter(),
		})
	}
}

impl<W: Write + Send + 'static> LogAdaptor for WriteLogger<W> {
	fn level(&self) -> LevelFilter {
		self.min_level
	}

	fn log(&self, record: &Record) {
		// TODO: change output

		self.writable
			.lock()
			.unwrap()
			.write_fmt(format_args!("{}", record.args()))
			.unwrap();

		todo!()
	}

	fn flush(&self) {
		let _ = self.writable.lock().unwrap().flush();
	}
}

struct DiscordLogger {
	min_level: LevelFilter,
}

impl DiscordLogger {
	fn boxed(level: Level) -> Box<Self> {
		Box::new(Self {
			min_level: level.to_level_filter(),
		})
	}
}

impl LogAdaptor for DiscordLogger {
	fn level(&self) -> LevelFilter {
		self.min_level
	}

	fn log(&self, record: &Record) {
		// TODO: send something
		todo!("send something")
	}

	fn flush(&self) {}
}

struct GlueLoggerConfig {
	crate_log_only: bool,
}

impl Default for GlueLoggerConfig {
	fn default() -> Self {
		Self {
			crate_log_only: false,
		}
	}
}

struct GlueLogger {
	adaptors: Vec<Box<dyn LogAdaptor>>,
	config: GlueLoggerConfig,
}

impl GlueLogger {
	fn new(adaptors: Vec<Box<dyn LogAdaptor>>, config: GlueLoggerConfig) -> Box<Self> {
		// env!("CARGO_PKG_NAME");

		Box::new(Self { adaptors, config })
	}
}

impl Log for GlueLogger {
	fn enabled(&self, meta: &Metadata) -> bool {
		self.adaptors
			.iter()
			.any(|adaptor| adaptor.level() >= meta.level())
	}

	fn log(&self, record: &Record) {
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

pub fn setup_logging() {
	let logger = GlueLogger::new(
		vec![
			TermLogger::boxed(Level::Warn),
			WriteLogger::boxed(
				File::create(format!(
					"logs/{}.log",
					SystemTime::now()
						.duration_since(SystemTime::UNIX_EPOCH)
						.expect("time went backwards")
						.as_millis()
				))
				.expect("failed to create log file"),
				Level::Info,
			),
			DiscordLogger::boxed(Level::Warn),
		],
		GlueLoggerConfig {
			crate_log_only: true,
		},
	);

	log::set_max_level(LevelFilter::Info);
	log::set_boxed_logger(logger).expect("failed to set logger");
}
