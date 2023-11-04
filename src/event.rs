use std::time::Duration;

use crossterm::event::{Event as TermEvent, EventStream, KeyEvent};
use futures::{future::FutureExt, StreamExt};
use tokio::sync::mpsc;

const FRAME_RATE: f64 = 2.0;

pub enum Event {
  Key(KeyEvent),
  Render,
}

pub struct Events {
  rx: mpsc::Receiver<Event>,
}

impl Events {
  pub async fn new() -> Events {
    let (tx, rx) = mpsc::channel(10);

    tokio::task::spawn(async move {
      let mut stream = EventStream::new();
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
    });

    Events { rx }
  }

  pub async fn next(&mut self) -> Option<Event> {
    self.rx.recv().await
  }
}
