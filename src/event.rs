use std::{io, sync::mpsc, thread, time::Duration};

use termion::{event::Key, input::TermRead};

pub enum Event<I> {
  Input(I),
  Tick,
}

pub struct Events {
  rx: mpsc::Receiver<Event<Key>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
  pub tick_rate: Duration,
}

impl Default for Config {
  fn default() -> Config {
    Config {
      tick_rate: Duration::from_millis(250),
    }
  }
}

impl Events {
  pub fn new() -> Events {
    let (tx, rx) = mpsc::channel();

    {
      let tx = tx.clone();

      thread::spawn(move || {
        let stdin = io::stdin();

        for key in stdin.keys().flatten() {
          if tx.send(Event::Input(key)).is_err() {
            return;
          }
        }
      })
    };

    thread::spawn(move || loop {
      let _ = tx.send(Event::Tick);

      thread::sleep(Duration::from_millis(250));
    });

    Events { rx }
  }

  pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
    self.rx.recv()
  }
}
