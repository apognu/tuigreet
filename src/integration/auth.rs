use libgreetd_stub::SessionOptions;

use super::common::IntegrationRunner;

#[tokio::test]
async fn authentication_ok() {
  let opts = SessionOptions {
    username: "apognu".to_string(),
    password: "password".to_string(),
    mfa: false,
  };

  let mut runner = IntegrationRunner::new(opts, None).await;

  let events = tokio::task::spawn({
    let mut runner = runner.clone();

    async move {
      runner.wait_until_buffer_contains("Username:").await;
      runner.send_text("apognu").await;
      runner.wait_until_buffer_contains("Password:").await;
      runner.send_text("password").await;
    }
  });

  runner.join_until_client_exit(events).await;
}

#[tokio::test]
async fn authentication_bad_password() {
  let opts = SessionOptions {
    username: "apognu".to_string(),
    password: "password".to_string(),
    mfa: false,
  };

  let mut runner = IntegrationRunner::new(opts, None).await;

  let events = tokio::task::spawn({
    let mut runner = runner.clone();

    {
      async move {
        runner.wait_until_buffer_contains("Username:").await;
        runner.send_text("apognu").await;
        runner.wait_until_buffer_contains("Password:").await;
        runner.send_text("password2").await;
        runner.wait_for_render().await;

        assert!(runner.output().await.contains("Authentication failed"));
      }
    }
  });

  runner.join_until_end(events).await;
}

#[tokio::test]
async fn authentication_ok_mfa() {
  let opts = SessionOptions {
    username: "apognu".to_string(),
    password: "password".to_string(),
    mfa: true,
  };

  let mut runner = IntegrationRunner::new(opts, None).await;

  let events = tokio::task::spawn({
    let mut runner = runner.clone();

    async move {
      runner.wait_until_buffer_contains("Username:").await;
      runner.send_text("apognu").await;
      runner.wait_until_buffer_contains("Password:").await;
      runner.send_text("password").await;
      runner.wait_until_buffer_contains("7 + 2 =").await;
      runner.send_text("9").await;
    }
  });

  runner.join_until_client_exit(events).await;
}

#[tokio::test]
async fn authentication_bad_mfa() {
  let opts = SessionOptions {
    username: "apognu".to_string(),
    password: "password".to_string(),
    mfa: true,
  };

  let mut runner = IntegrationRunner::new(opts, None).await;

  let events = tokio::task::spawn({
    let mut runner = runner.clone();

    async move {
      runner.wait_until_buffer_contains("Username:").await;
      runner.send_text("apognu").await;
      runner.wait_until_buffer_contains("Password:").await;
      runner.send_text("password").await;
      runner.wait_until_buffer_contains("7 + 2 =   ").await;
      runner.send_text("10").await;
      runner.wait_for_render().await;

      assert!(runner.output().await.contains("Authentication failed"));
      assert!(runner.output().await.contains("Password:"));
    }
  });

  runner.join_until_end(events).await;
}
