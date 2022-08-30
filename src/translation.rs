use crate::{states::Command, Context};
use anyhow::{anyhow, Result};
use fluent::{bundle, FluentArgs, FluentResource};
use fluent_syntax::ast::Pattern;
use intl_memoizer::concurrent::IntlLangMemoizer as ConcurrentIntlLangMemoizer;
use std::{
	collections::HashMap,
	fs::{read_dir, read_to_string},
	path::Path,
};
use unic_langid::LanguageIdentifier;

/// The concurrent Fluent bundle used to cache the language results
type FluentBundle = bundle::FluentBundle<FluentResource, ConcurrentIntlLangMemoizer>;

/// Manages the client internationalization
pub struct Translations {
	/// The fallback locale
	fallback: LanguageIdentifier,
	/// The available locales
	bundles: HashMap<LanguageIdentifier, FluentBundle>,
}

/// Reads and parses the given Fluent file
fn read_fluent_file(path: &Path) -> Result<(LanguageIdentifier, FluentBundle)> {
	// Extract locale from filename
	let locale: LanguageIdentifier = path
		.file_stem()
		.ok_or(anyhow!("Invalid `.ftl` file"))?
		.to_str()
		.ok_or(anyhow!("Invalid UTF-8 filename"))?
		.parse()?;

	// Load .ftl resource
	let file_contents = read_to_string(&path)?;
	let resource = FluentResource::try_new(file_contents)
		.map_err(|(_, e)| anyhow!("failed to parse {:?}: {:?}", path, e))?;

	// Associate .ftl resource with locale and bundle it
	let mut bundle = FluentBundle::new_concurrent(vec![locale.clone()]);
	bundle
		.add_resource(resource)
		.map_err(|e| anyhow!("failed to add resource to bundle: {:?}", e))?;

	Ok((locale, bundle))
}

impl Translations {
	/// Load all available translations from the given directory
	pub(crate) fn from_folder(folder: &str, fallback: LanguageIdentifier) -> Result<Self> {
		let bundles = read_dir(folder)?
			.map(Result::unwrap)
			.filter(|file| matches!(file.path().extension(), Some(ext) if ext == "ftl"))
			.map(|file| read_fluent_file(&file.path()))
			.collect::<Result<_, _>>()?;

		Ok(Self { fallback, bundles })
	}

	/// Apply translations to the given commands
	pub(crate) fn apply_interaction_translations(&self, commands: &mut [Command]) {
		for command in &mut *commands {
			// Skip prefix commands
			if command.prefix_action.is_some() {
				continue;
			}

			for (locale, bundle) in &self.bundles {
				let msg = match bundle.get_message(&command.name) {
					Some(x) => x,
					None => {
						tracing::error!(
							"missing command translation {} for locale {}",
							command.name,
							locale
						);
						continue;
					}
				};

				command.name_localizations.insert(
					locale.to_string(),
					format(bundle, msg.value().unwrap(), None),
				);

				command.description_localizations.insert(
					locale.to_string(),
					format(
						bundle,
						msg.get_attribute("description").unwrap().value(),
						None,
					),
				);

				for parameter in &mut command.parameters {
					parameter.name_localizations.insert(
						locale.to_string(),
						format(
							bundle,
							msg.get_attribute(&parameter.name).unwrap().value(),
							None,
						),
					);

					parameter.description_localizations.insert(
						locale.to_string(),
						format(
							bundle,
							msg.get_attribute(&format!("{}-description", parameter.name))
								.unwrap()
								.value(),
							None,
						),
					);

					for choice in &mut parameter.choices {
						choice.localizations.insert(
							locale.to_string(),
							format(
								bundle,
								bundle.get_message(&choice.name).unwrap().value().unwrap(),
								None,
							),
						);
					}
				}
			}
		}
	}
}

/// Formats the given message with the given arguments
fn format(bundle: &FluentBundle, pattern: &Pattern<&str>, args: Option<&FluentArgs>) -> String {
	let mut errors = Vec::new();

	let formatted = bundle
		.format_pattern(pattern, args, &mut errors)
		.into_owned();

	for error in errors {
		tracing::error!("fluent format pattern error {}", error);
	}

	formatted
}

pub(crate) trait Translate {
	fn get(&self, key: &str, args: Option<&FluentArgs>) -> Result<String>;
}

impl Translate for Context<'_> {
	fn get(&self, key: &str, args: Option<&FluentArgs>) -> Result<String> {
		let translations = &self.data().translations;
		let locale: LanguageIdentifier = self.locale().unwrap().parse().unwrap();

		if let Some(bundle) = translations.bundles.get(&locale) {
			Ok(format(
				bundle,
				bundle.get_message(key).unwrap().value().unwrap(),
				args,
			))
		} else if let Some(bundle) = translations.bundles.get(&translations.fallback) {
			Ok(format(
				bundle,
				bundle.get_message(key).unwrap().value().unwrap(),
				args,
			))
		} else {
			tracing::warn!("unknown fluent key `{}`", key);
			Ok(key.to_owned())
		}
	}
}
