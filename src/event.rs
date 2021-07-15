use std::{io, time::Duration};

use termion::{event::Key, input::TermRead};
use tokio::sync::mpsc;

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
  pub async fn new() -> Events {
    let (tx, rx) = mpsc::channel(10);

    {
      let tx = tx.clone();

      tokio::task::spawn(async move {
        let stdin = io::stdin();

        for key in stdin.keys().flatten() {
          if tx.send(Event::Input(key)).await.is_err() {
            return;
          }
        }
      })
    };

    tokio::task::spawn(async move {
      loop {
        let _ = tx.send(Event::Tick).await;

        tokio::time::sleep(Duration::from_millis(250)).await;
      }
    });

    Events { rx }
  }

  pub async fn next(&mut self) -> Option<Event<Key>> {
    self.rx.recv().await
  }
}
