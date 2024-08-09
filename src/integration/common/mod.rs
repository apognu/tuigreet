mod backend;
mod output;

use std::{
  panic,
  sync::{Arc, Mutex},
  time::Duration,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use libgreetd_stub::SessionOptions;
use tempfile::NamedTempFile;
use tokio::{
  sync::{
    mpsc::{Receiver, Sender},
    RwLock,
  },
  task::{JoinError, JoinHandle},
};
use tui::buffer::Buffer;

use crate::{
  event::{Event, Events},
  ui::sessions::SessionSource,
  Greeter,
};

pub(super) use self::{
  backend::{output, TestBackend},
  output::*,
};

pub(super) struct IntegrationRunner(Arc<RwLock<_IntegrationRunner>>);

struct _IntegrationRunner {
  server: Option<JoinHandle<()>>,
  client: Option<JoinHandle<()>>,

  pub buffer: Arc<Mutex<Buffer>>,
  pub sender: Sender<Event>,
  pub tick: Receiver<bool>,
}

impl Clone for IntegrationRunner {
  fn clone(&self) -> Self {
    IntegrationRunner(Arc::clone(&self.0))
  }
}

impl IntegrationRunner {
  pub async fn new(opts: SessionOptions, builder: Option<fn(&mut Greeter)>) -> IntegrationRunner {
    IntegrationRunner::new_with_size(opts, builder, (200, 40)).await
  }

  pub async fn new_with_size(opts: SessionOptions, builder: Option<fn(&mut Greeter)>, size: (u16, u16)) -> IntegrationRunner {
    let socket = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();

    let (backend, buffer, tick) = TestBackend::new(size.0, size.1);
    let events = Events::new().await;
    let sender = events.sender();

    let server = tokio::task::spawn({
      let socket = socket.clone();

      async move {
        libgreetd_stub::start(&socket, &opts).await;
      }
    });

    let client = tokio::task::spawn(async move {
      let mut greeter = Greeter::new(events.sender()).await;
      greeter.session_source = SessionSource::Command("uname".to_string());

      if let Some(builder) = builder {
        builder(&mut greeter);
      }

      if greeter.config.is_none() {
        greeter.config = Greeter::options().parse(&[""]).ok();
      }

      greeter.logfile = "/tmp/tuigreet.log".to_string();
      greeter.socket = socket.to_str().unwrap().to_string();
      greeter.events = Some(events.sender());
      greeter.connect().await;

      let _ = crate::run(backend, greeter, events).await;
    });

    IntegrationRunner(Arc::new(RwLock::new(_IntegrationRunner {
      server: Some(server),
      client: Some(client),
      buffer,
      sender,
      tick,
    })))
  }

  pub async fn join_until_client_exit(&mut self, mut events: JoinHandle<()>) {
    let (mut server, mut client) = {
      let mut runner = self.0.write().await;

      (runner.server.take().unwrap(), runner.client.take().unwrap())
    };

    let mut exited = false;

    while !exited {
      tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(5)) => break,
        _ = (&mut server) => {}
        _ = (&mut client) => { exited = true; },
        ret = (&mut events), if !events.is_finished() => rethrow(ret),
      }
    }

    assert!(exited, "tuigreet did not exit");
  }

  pub async fn join_until_end(&mut self, events: JoinHandle<()>) {
    let (server, client) = {
      let mut runner = self.0.write().await;

      (runner.server.take().unwrap(), runner.client.take().unwrap())
    };

    tokio::select! {
      _ = tokio::time::sleep(Duration::from_secs(5)) => {},
      _ = server => {}
      _ = client => {},
      ret = events => rethrow(ret),
    }
  }

  #[allow(unused)]
  pub async fn wait_until_buffer_contains(&mut self, needle: &str) {
    loop {
      if output(&self.0.read().await.buffer).contains(needle) {
        return;
      }

      self.wait_for_render().await;
    }
  }

  #[allow(unused, unused_must_use)]
  pub async fn send_key(&self, key: KeyCode) {
    self.0.write().await.sender.send(Event::Key(KeyEvent::new(key, KeyModifiers::empty()))).await;
  }

  #[allow(unused, unused_must_use)]
  pub async fn send_modified_key(&self, key: KeyCode, modifiers: KeyModifiers) {
    self.0.write().await.sender.send(Event::Key(KeyEvent::new(key, modifiers))).await;
  }

  #[allow(unused, unused_must_use)]
  pub async fn send_text(&self, text: &str) {
    for char in text.chars() {
      self.0.write().await.sender.send(Event::Key(KeyEvent::new(KeyCode::Char(char), KeyModifiers::empty()))).await;
    }

    self.0.write().await.sender.send(Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))).await;
  }

  #[allow(unused)]
  pub async fn wait_for_render(&mut self) {
    self.0.write().await.tick.recv().await;
  }

  pub async fn output(&self) -> Output {
    Output(output(&self.0.read().await.buffer))
  }
}

fn rethrow(result: Result<(), JoinError>) {
  if let Err(err) = result {
    if let Ok(panick) = err.try_into_panic() {
      panic::resume_unwind(panick);
    }
  }
}
