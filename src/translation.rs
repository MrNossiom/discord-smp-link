//! Fluent Project translation system

use crate::states::{ApplicationContext, Command, Context};
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
		.ok_or_else(|| anyhow!("Invalid `.ftl` file"))?
		.to_str()
		.ok_or_else(|| anyhow!("Invalid UTF-8 filename"))?
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
				let command_translation = match bundle.get_message(&command.name) {
					Some(x) => x,
					None => {
						tracing::error!(
							"translation for command `{}` with locale `{}` does not exist",
							command.name,
							locale
						);

						continue;
					}
				};

				match command_translation.value() {
					Some(name) => {
						command
							.name_localizations
							.insert(locale.to_string(), format(bundle, name, None));
					}
					None => {
						tracing::error!(
							"translation for command `{}` with locale `{}` does not have a name",
							command.name,
							locale
						);
					}
				}

				match command_translation.get_attribute("description") {
					Some(description) => {
						command.description_localizations.insert(
							locale.to_string(),
							format(bundle, description.value(), None),
						);
					}
					None => {
						tracing::error!(
							"translation for command `{}` with locale `{}` does not have a description",
							command.name,
							locale
						);
					}
				}

				for parameter in &mut command.parameters {
					match command_translation.get_attribute(&parameter.name) {
						Some(param_name) => {
							parameter.name_localizations.insert(
								locale.to_string(),
								format(bundle, param_name.value(), None),
							);
						}
						None => {
							tracing::error!(
								"translation for command `{}` with locale `{}` does not have a name for the parameter `{}`",
								command.name,
								locale,
								parameter.name
							);
						}
					}

					match command_translation
						.get_attribute(&format!("{}-description", parameter.name))
					{
						Some(param_description) => {
							parameter.description_localizations.insert(
								locale.to_string(),
								format(bundle, param_description.value(), None),
							);
						}
						None => {
							tracing::error!(
								"translation for command `{}` with locale `{}` does not have a description for the parameter `{}`",
								command.name,
								locale,
								parameter.name
							);
						}
					}

					for choice in &mut parameter.choices {
						match command_translation.get_attribute(&format!("{}-choice", choice.name))
						{
							Some(choice_name) => {
								parameter.description_localizations.insert(
									locale.to_string(),
									format(bundle, choice_name.value(), None),
								);
							}
							None => {
								tracing::error!(
									"translation for command `{}` with locale `{}` does not have a translation for the choice `{}`",
									command.name,
									locale,
									choice.name
								);
							}
						}
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

/// Trait for client internationalisation
pub(crate) trait Translate {
	/// Get the translation for the given message with a locale provided by self context
	fn get_checked(&self, key: &str, args: Option<&FluentArgs>) -> Result<String>;

	/// Get a translated key of the key itself in case it is not found
	fn get(&self, key: &str, args: Option<&FluentArgs>) -> String {
		match self.get_checked(key, args) {
			Ok(string) => string,
			Err(error) => {
				tracing::error!("error for key {key} with args {args:?}: {error}");
				key.to_owned()
			}
		}
	}
}

impl Translate for ApplicationContext<'_> {
	fn get_checked(&self, key: &str, args: Option<&FluentArgs>) -> Result<String> {
		let translations = &self.data.translations;
		let locale: LanguageIdentifier = self.interaction.locale().parse()?;

		let bundle = translations.bundles.get(&locale).unwrap_or_else(|| {
			translations
				.bundles
				.get(&translations.fallback)
				.expect("failed to load fallback locale bundle")
		});

		match bundle.get_message(key) {
			Some(message) => match message.value() {
				Some(pattern) => Ok(format(bundle, pattern, args)),
				None => Err(anyhow!("message `{}` has no value", key)),
			},
			None => Err(anyhow!("unknown fluent key `{}`", key)),
		}
	}
}

impl Translate for Context<'_> {
	fn get_checked(&self, key: &str, args: Option<&FluentArgs>) -> Result<String> {
		let translations = &self.data().translations;
		let locale: LanguageIdentifier = match self.locale() {
			Some(locale) => locale.parse()?,
			None => translations.fallback.clone(),
		};

		let bundle = translations.bundles.get(&locale).unwrap_or_else(|| {
			translations
				.bundles
				.get(&translations.fallback)
				.expect("failed to load fallback locale bundle")
		});

		match bundle.get_message(key) {
			Some(message) => match message.value() {
				Some(pattern) => Ok(format(bundle, pattern, args)),
				None => Err(anyhow!("message `{}` has no value", key)),
			},
			None => Err(anyhow!("unknown fluent key `{}`", key)),
		}
	}
}
