#[macro_use]
extern crate smart_default;

#[macro_use]
mod macros;

mod event;
mod greeter;
mod info;
mod ipc;
mod keyboard;
mod power;
mod ui;

use std::{error::Error, io, process, sync::Arc};

use crossterm::{
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};
use greetd_ipc::Request;
use tokio::sync::RwLock;
use tui::{backend::CrosstermBackend, Terminal};

pub use self::greeter::*;
use self::{event::Events, ipc::Ipc};

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
  let mut stdout = io::stdout();

  enable_raw_mode()?;
  execute!(stdout, EnterAlternateScreen)?;

  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  terminal.clear()?;

  let mut events = Events::new().await;
  let ipc = Ipc::new();

  if greeter.remember && !greeter.username.is_empty() {
    ipc.send(Request::CreateSession { username: greeter.username.clone() }).await;
  }

  let greeter = Arc::new(RwLock::new(greeter));

  tokio::task::spawn({
    let greeter = greeter.clone();
    let mut ipc = ipc.clone();

    async move {
      loop {
        let _ = ipc.handle(greeter.clone()).await;
      }
    }
  });

  tokio::task::spawn({
    let greeter = greeter.clone();

    async move {
      loop {
        let command = greeter.write().await.power_command.take();

        if let Some(command) = command {
          power::run(&greeter, command).await;
        }
      }
    }
  });

  loop {
    if let Some(status) = greeter.read().await.exit {
      return Err(status.into());
    }

    ui::draw(greeter.clone(), &mut terminal).await?;
    keyboard::handle(greeter.clone(), &mut events, ipc.clone()).await?;
  }
}

pub async fn exit(mut greeter: &mut Greeter, status: AuthStatus) {
  match status {
    AuthStatus::Success => {}
    AuthStatus::Cancel | AuthStatus::Failure => Ipc::cancel(&mut greeter).await,
  }

  clear_screen();
  let _ = disable_raw_mode();

  greeter.exit = Some(status);
}

pub fn clear_screen() {
  let backend = CrosstermBackend::new(io::stdout());

  if let Ok(mut terminal) = Terminal::new(backend) {
    let _ = terminal.clear();
  }
}

#[cfg(debug_assertions)]
pub fn log(msg: &str) {
  use std::io::Write;

  let time = chrono::Utc::now();

  let mut file = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/tuigreet.log").unwrap();
  file.write_all(format!("{:?} - ", time).as_ref()).unwrap();
  file.write_all(msg.as_ref()).unwrap();
  file.write_all("\n".as_bytes()).unwrap();
}
