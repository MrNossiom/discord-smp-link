//! Custom loggers with their output adaptors

use log::{Level, LevelFilter, Log, Metadata, Record};
use std::{fs::File, io::Write, sync::Mutex, time::SystemTime};

struct TermLogger {
	/// Min and Max log levels
	levels: (LevelFilter, LevelFilter),
}

impl TermLogger {
	fn boxed(levels: (Level, Level)) -> Box<Self> {
		Box::new(Self {
			levels: (levels.0.to_level_filter(), levels.1.to_level_filter()),
		})
	}
}

impl Log for TermLogger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		self.levels.0 >= metadata.level() && self.levels.1 <= metadata.level()
	}

	fn log(&self, record: &Record) {
		// TODO
		println!(
			"{} - {} - {}",
			record.target(),
			record.module_path().unwrap_or_default(),
			record.args()
		);
	}

	fn flush(&self) {
		// TODO: impl
		todo!("flush something")
	}
}

struct WriteLogger<W: Write + Send + 'static> {
	writable: Mutex<W>,
	/// Min and Max log levels
	levels: (LevelFilter, LevelFilter),
}

impl<W: Write + Send + 'static> WriteLogger<W> {
	fn boxed(writable: W, levels: (Level, Level)) -> Box<Self> {
		Box::new(Self {
			writable: Mutex::new(writable),
			levels: (levels.0.to_level_filter(), levels.1.to_level_filter()),
		})
	}
}

impl<W: Write + Send + 'static> Log for WriteLogger<W> {
	fn enabled(&self, metadata: &Metadata) -> bool {
		self.levels.0 >= metadata.level() && self.levels.1 <= metadata.level()
	}
	fn log(&self, record: &Record) {
		// TODO: change output

		writeln!(
			self.writable.lock().unwrap(),
			"{} - {} - {}",
			record.target(),
			record.module_path().unwrap_or_default(),
			record.args()
		)
		.unwrap();
	}

	fn flush(&self) {
		let _ = self.writable.lock().unwrap().flush();
	}
}

struct DiscordLogger {
	/// Min and Max log levels
	levels: (LevelFilter, LevelFilter),
}

impl DiscordLogger {
	fn boxed(levels: (Level, Level)) -> Box<Self> {
		Box::new(Self {
			levels: (levels.0.to_level_filter(), levels.1.to_level_filter()),
		})
	}
}

impl Log for DiscordLogger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		self.levels.0 >= metadata.level() && self.levels.1 <= metadata.level()
	}

	fn log(&self, record: &Record) {
		// TODO: send something
		// todo!("send something");
	}

	fn flush(&self) {}
}

#[derive(Default)]
struct GlueLoggerConfig {
	crate_log_only: bool,
}

struct GlueLogger {
	adaptors: Vec<Box<dyn Log>>,
	config: GlueLoggerConfig,
	crate_name: &'static str,
}

impl GlueLogger {
	fn boxed(adaptors: Vec<Box<dyn Log>>, config: GlueLoggerConfig) -> Box<Self> {
		let crate_name = env!("CARGO_PKG_NAME");

		Box::new(Self {
			adaptors,
			config,
			crate_name,
		})
	}
}

impl Log for GlueLogger {
	fn enabled(&self, meta: &Metadata) -> bool {
		self.adaptors.iter().any(|adaptor| adaptor.enabled(meta))
	}

	fn log(&self, record: &Record) {
		if let Some(path) = record.module_path() {
			if !path.starts_with(self.crate_name) {
				return;
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

pub fn setup_logging() {
	let logger = GlueLogger::boxed(
		vec![
			TermLogger::boxed((Level::Info, Level::Error)),
			WriteLogger::boxed(
				File::create(format!(
					"logs/{}.log",
					SystemTime::now()
						.duration_since(SystemTime::UNIX_EPOCH)
						.expect("time went backwards")
						.as_millis()
				))
				.expect("failed to create log file"),
				(Level::Info, Level::Error),
			),
			DiscordLogger::boxed((Level::Warn, Level::Error)),
		],
		GlueLoggerConfig {
			crate_log_only: true,
		},
	);

	log::set_max_level(LevelFilter::Trace);
	log::set_boxed_logger(logger).expect("failed to set logger");
}
