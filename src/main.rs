mod config;
mod event;
mod info;
mod ipc;
mod keyboard;
mod ui;

use std::{error::Error, io, process};

use termion::raw::IntoRawMode;
use tui::{backend::TermionBackend, Terminal};

pub use self::config::*;
use self::event::Events;

fn main() {
  if let Err(error) = run() {
    if let Some(status) = error.downcast_ref::<AuthStatus>() {
      if let AuthStatus::Success = *status {
        return;
      }

      process::exit(1);
    } else {
      process::exit(1);
    }
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  let mut greeter = config::parse_options(Greeter::new()?);

  let stdout = io::stdout().into_raw_mode()?;
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  terminal.clear()?;

  let events = Events::new();

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

  Err(status)
}
