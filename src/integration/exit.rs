use crossterm::event::{KeyCode, KeyModifiers};
use libgreetd_stub::SessionOptions;

use super::common::IntegrationRunner;

#[tokio::test]
async fn exit() {
  let opts = SessionOptions {
    username: "apognu".to_string(),
    password: "password".to_string(),
    mfa: false,
  };

  let mut runner = IntegrationRunner::new(opts, None).await;

  let events = tokio::task::spawn({
    let mut runner = runner.clone();

    async move {
      runner.send_modified_key(KeyCode::Char('x'), KeyModifiers::CONTROL).await;
      runner.wait_for_render().await;
    }
  });

  runner.join_until_client_exit(events).await;
}
