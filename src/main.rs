mod config;
mod event;
mod info;
mod ipc;
mod keyboard;
#[macro_use]
mod macros;
mod ui;

use std::{error::Error, io, process};

use greetd_ipc::Request;
use i18n_embed::{
  fluent::{fluent_language_loader, FluentLanguageLoader},
  DesktopLanguageRequester, LanguageLoader,
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use termion::raw::IntoRawMode;
use tui::{backend::TermionBackend, Terminal};

pub use self::config::*;
use self::event::Events;

#[derive(RustEmbed)]
#[folder = "contrib/locales"]
struct Localizations;

lazy_static! {
  static ref MESSAGES: FluentLanguageLoader = {
    let locales = Localizations;
    let loader = fluent_language_loader!();
    loader.load_languages(&locales, &[loader.fallback_language()]).unwrap();

    let _ = i18n_embed::select(&loader, &locales, &DesktopLanguageRequester::requested_languages());

    loader
  };
}

fn main() {
  if let Err(error) = run() {
    if let Some(AuthStatus::Success) = error.downcast_ref::<AuthStatus>() {
      return;
    }

    process::exit(1);
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  let mut greeter = Greeter::new();

  let stdout = io::stdout().into_raw_mode()?;
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  terminal.clear()?;

  let events = Events::new();

  if greeter.remember && !greeter.username.is_empty() {
    greeter.request = Some(Request::CreateSession { username: greeter.username.clone() });
  }

  loop {
    ui::draw(&mut terminal, &mut greeter)?;
    ipc::handle(&mut greeter)?;
    keyboard::handle(&mut greeter, &events)?;
  }
}

pub fn exit(greeter: &mut Greeter, status: AuthStatus) -> Result<(), AuthStatus> {
  match status {
    AuthStatus::Success => {}
    AuthStatus::Cancel | AuthStatus::Failure => ipc::cancel(greeter),
  }

  clear_screen();

  Err(status)
}

pub fn clear_screen() {
  let backend = TermionBackend::new(io::stdout());

  if let Ok(mut terminal) = Terminal::new(backend) {
    let _ = terminal.clear();
  }
}
