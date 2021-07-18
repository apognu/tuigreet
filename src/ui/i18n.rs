use i18n_embed::{
  fluent::{fluent_language_loader, FluentLanguageLoader},
  DesktopLanguageRequester, LanguageLoader,
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "contrib/locales"]
struct Localizations;

lazy_static! {
  pub static ref MESSAGES: FluentLanguageLoader = {
    let locales = Localizations;
    let loader = fluent_language_loader!();
    loader.load_languages(&locales, &[loader.fallback_language()]).unwrap();

    let _ = i18n_embed::select(&loader, &locales, &DesktopLanguageRequester::requested_languages());

    loader
  };
}
