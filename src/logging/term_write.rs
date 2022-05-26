//! Output adaptors for terminal and [`Write`] trait implementors

use super::ToPrettyRecord;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::{io::Write, sync::Mutex};

/// An adaptor that writes to anything that implement [`Write`]
pub struct WriteAdaptor<W: Write + Send + 'static> {
	/// The writer to write to
	writable: Mutex<W>,
	/// Min and Max log levels
	min_level: LevelFilter,
}

impl<W: Write + Send + 'static> WriteAdaptor<W> {
	/// Create a new boxed [`WriteAdaptor`]
	pub fn boxed(writable: W, level: Level) -> Box<Self> {
		Box::new(Self {
			writable: Mutex::new(writable),
			min_level: level.to_level_filter(),
		})
	}
}

impl<W: Write + Send + 'static> Log for WriteAdaptor<W> {
	fn enabled(&self, metadata: &Metadata) -> bool {
		self.min_level >= metadata.level()
	}

	fn log(&self, record: &Record) {
		if !self.enabled(record.metadata()) {
			return;
		}

		let mut writable = self.writable.lock().expect("lock poisoned");

		writeln!(writable, "{}", record.to_pretty_record()).unwrap();
	}

	fn flush(&self) {
		let _ = self.writable.lock().expect("lock poisoned").flush();
	}
}

/// An adaptor that write to `stdout` and `stderr` for error levels
pub struct TermAdaptor {
	/// Minimum log level
	min_level: LevelFilter,
}

impl TermAdaptor {
	/// Create a new boxed [`TermAdaptor`]
	pub fn boxed(min_level: Level) -> Box<Self> {
		Box::new(Self {
			min_level: min_level.to_level_filter(),
		})
	}
}

impl Log for TermAdaptor {
	fn enabled(&self, metadata: &Metadata) -> bool {
		self.min_level >= metadata.level()
	}

	fn log(&self, record: &Record) {
		if !self.enabled(record.metadata()) {
			return;
		}

		// TODO: output to stderr or stdout
		println!("{}", record.to_pretty_record_color());
	}

	fn flush(&self) {
		// TODO: implement
	}
}
