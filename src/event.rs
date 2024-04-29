use std::time::Duration;

use crossterm::event::{Event as TermEvent, KeyEvent};
use futures::{future::FutureExt, StreamExt};
use tokio::{
  process::Command,
  sync::mpsc::{self, Sender},
};

#[cfg(not(test))]
use crossterm::event::EventStream;

use crate::AuthStatus;

const FRAME_RATE: f64 = 2.0;

pub enum Event {
  Key(KeyEvent),
  Render,
  PowerCommand(Command),
  Exit(AuthStatus),
}

pub struct Events {
  rx: mpsc::Receiver<Event>,
  tx: mpsc::Sender<Event>,
}

impl Events {
  pub async fn new() -> Events {
    let (tx, rx) = mpsc::channel(10);

    tokio::task::spawn({
      let tx = tx.clone();

      async move {
        #[cfg(not(test))]
        let mut stream = EventStream::new();

        // In tests, we are not capturing events from the terminal, so we need
        // to replace the crossterm::EventStream with a dummy pending stream.
        #[cfg(test)]
        let mut stream = futures::stream::pending::<Result<TermEvent, ()>>();

        let mut render_interval = tokio::time::interval(Duration::from_secs_f64(1.0 / FRAME_RATE));

        loop {
          let render = render_interval.tick();
          let event = stream.next().fuse();

          tokio::select! {
            event = event => {
              if let Some(Ok(TermEvent::Key(event))) = event {
                let _ = tx.send(Event::Key(event)).await;
                let _ = tx.send(Event::Render).await;
              }
            }

            _ = render => { let _ = tx.send(Event::Render).await; },
          }
        }
      }
    });

    Events { rx, tx }
  }

  pub async fn next(&mut self) -> Option<Event> {
    self.rx.recv().await
  }

  pub fn sender(&self) -> Sender<Event> {
    self.tx.clone()
  }
}
