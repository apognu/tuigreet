use std::time::Duration;

use crossterm::event::{Event as TermEvent, EventStream, KeyEvent};
use futures::{future::FutureExt, StreamExt};
use tokio::sync::mpsc;

const TICK_RATE: u64 = 250;

pub enum Event {
  Key(KeyEvent),
  Tick,
}

pub struct Events {
  rx: mpsc::Receiver<Event>,
}

impl Events {
  pub async fn new() -> Events {
    let (tx, rx) = mpsc::channel(10);

    tokio::task::spawn(async move {
      let mut stream = EventStream::new();
      let mut interval = tokio::time::interval(Duration::from_millis(TICK_RATE));

      loop {
        let delay = interval.tick();
        let event = stream.next().fuse();

        tokio::select! {
          event = event => {
            if let Some(Ok(TermEvent::Key(event))) = event {
              let _ = tx.send(Event::Key(event)).await;
            }
          }

          _ = delay => { let _ = tx.send(Event::Tick).await; },
        }
      }
    });

    Events { rx }
  }

  pub async fn next(&mut self) -> Option<Event> {
    self.rx.recv().await
  }
}
