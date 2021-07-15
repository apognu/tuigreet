#[macro_use]
extern crate smart_default;

#[macro_use]
mod macros;

mod config;
mod event;
mod info;
mod ipc;
mod keyboard;
mod ui;

use std::{error::Error, io, process, sync::Arc};

use greetd_ipc::Request;
use i18n_embed::{
  fluent::{fluent_language_loader, FluentLanguageLoader},
  DesktopLanguageRequester, LanguageLoader,
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use termion::raw::IntoRawMode;
use tokio::sync::RwLock;
use tui::{backend::TermionBackend, Terminal};

pub use self::config::*;
use self::{event::Events, ipc::new_ipc};

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

#[tokio::main]
async fn main() {
  if let Err(error) = run().await {
    if let Some(AuthStatus::Success) = error.downcast_ref::<AuthStatus>() {
      return;
    }

    process::exit(1);
  }
}

async fn run() -> Result<(), Box<dyn Error>> {
  let greeter = Greeter::new().await;

  let stdout = io::stdout().into_raw_mode()?;
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  terminal.clear()?;

  let mut events = Events::new().await;

  let (ipc_rx, ipc_tx) = new_ipc();

  if greeter.remember && !greeter.username.is_empty() {
    let _ = ipc_tx.lock().await.send(Request::CreateSession { username: greeter.username.clone() }).await;
  }

  let greeter = Arc::new(RwLock::new(greeter));

  tokio::task::spawn({
    let greeter = greeter.clone();
    let (ipc_rx, ipc_tx) = (ipc_rx.clone(), ipc_tx.clone());

    async move {
      loop {
        let _ = ipc::handle(greeter.clone(), ipc_tx.clone(), ipc_rx.clone()).await;
      }
    }
  });

  loop {
    greeter.read().await.exit?;

    ui::draw(greeter.clone(), &mut terminal).await?;
    keyboard::handle(greeter.clone(), &mut events, ipc_tx.clone()).await?;
  }
}

pub async fn exit(mut greeter: &mut Greeter, status: AuthStatus) {
  match status {
    AuthStatus::Success => {}
    AuthStatus::Cancel | AuthStatus::Failure => ipc::cancel(&mut greeter).await,
  }

  clear_screen();

  greeter.exit = Err(status);
}

pub fn clear_screen() {
  let backend = TermionBackend::new(io::stdout());

  if let Ok(mut terminal) = Terminal::new(backend) {
    let _ = terminal.clear();
  }
}

#[cfg(debug_assertions)]
pub fn log(msg: &str) {
  use std::io::Write;

  let mut file = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/tuigreet.log").unwrap();
  file.write_all(msg.as_ref()).unwrap();
  file.write_all("\n".as_bytes()).unwrap();
}
