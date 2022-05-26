use super::ToPrettyRecord;
use crate::states::STATE;
use anyhow::Result;
use futures::future::join_all;
use log::{Level, LevelFilter, Log, Metadata, Record};
use poise::serenity_prelude::{self as serenity, ExecuteWebhook};
use std::{
	borrow::Cow,
	sync::{Arc, Mutex},
	time::Duration,
};
use tokio::{spawn, time::interval};

/// An adaptor that sends messages to a `Discord` webhook
pub struct DiscordAdaptor {
	/// Min and Max log levels
	min_level: LevelFilter,
}

impl DiscordAdaptor {
	/// Create a new boxed [`DiscordAdaptor`]
	pub fn boxed(level: Level) -> Box<Self> {
		Box::new(Self {
			min_level: level.to_level_filter(),
		})
	}
}

impl Log for DiscordAdaptor {
	fn enabled(&self, metadata: &Metadata) -> bool {
		self.min_level >= metadata.level()
	}

	fn log(&self, record: &Record) {
		if !self.enabled(record.metadata()) {
			return;
		}

		STATE
			.webhook
			.queue_message(None, record.to_pretty_record().clone().as_str());
	}

	fn flush(&self) {}
}

/// A trait to make a task repeat
pub trait Looper {
	/// The interval between each task
	const INTERVAL: u64;

	/// Start the task looper
	fn start(self: Arc<Self>)
	where
		Self: Sync,
	{
		let mut interval = interval(Duration::from_millis(Self::INTERVAL));

		loop {
			spawn(async move { interval.tick().await });

			if let Err(err) = self.task() {
				log::error!("{}", err);
			}
		}
	}

	/// The task to loop over
	fn task(&self) -> Result<()>;
}

pub struct WebhookLogs<'a> {
	/// A http client to make discord requests
	pub http: serenity::Http,
	/// The webhook to send logs to
	webhook: serenity::Webhook,

	logs_queue: Mutex<Vec<&'a str>>,
	message_queue: Mutex<Vec<ExecuteWebhook<'a>>>,
}

impl<'a> WebhookLogs<'a> {
	pub fn new(http: serenity::Http, webhook: serenity::Webhook) -> Self {
		Self {
			http,
			webhook,
			logs_queue: Default::default(),
			message_queue: Default::default(),
		}
	}

	pub async fn queue<F>(&self, builder: F)
	where
		for<'b> F: FnOnce(&'b mut ExecuteWebhook<'a>) -> &'b mut ExecuteWebhook<'a>,
	{
		let mut execute_webhook = ExecuteWebhook::default();

		builder(&mut execute_webhook);

		self.message_queue
			.lock()
			.expect("lock poisoned")
			.push(execute_webhook);
	}

	pub async fn queue_message(&self, level: Option<Level>, message: &'a str) {
		self.logs_queue.lock().expect("lock poisoned").push(message);
	}
}

impl Looper for WebhookLogs<'_> {
	const INTERVAL: u64 = 1100;

	fn task(&self) -> Result<()> {
		let pending_logs = self.logs_queue.lock().expect("lock poisoned").drain(..);
		let pending_message = self.message_queue.lock().expect("lock poisoned").drain(..);

		// TODO: crate chunks to send for network optimization
		let mut chunks: Vec<Cow<'_, str>> = Vec::with_capacity(pending_message.len());

		for log in pending_logs {
			if let Some(chunk) = chunks.last_mut() {
				if chunk.len() + log.len() > 2000 {
					chunks.push(Cow::Borrowed(log));
				} else {
					chunk.to_mut().push_str(log);
				}
			} else {
				chunks.push(Cow::Borrowed(log));
			}

			let logs_chunks = chunks
				.iter()
				.map(|chunk| {
					self.webhook
						.execute(&self.http, false, |b| b.content(chunk))
				})
				.collect::<Vec<_>>();

			spawn(async move {
				join_all(logs_chunks).await;
			});
		}

		{
			let messages = pending_message.into_iter().map(|message| {
				self.webhook.execute(&self.http, false, |b| {
					*b = message;
					b
				})
			});

			spawn(async move {
				join_all(messages).await;
			});
		}

		Ok(())
	}
}
