use std::{error::Error, sync::Arc};

use greetd_ipc::{codec::TokioCodec, AuthMessageType, ErrorType, Request, Response};
use tokio::sync::{
  mpsc::{Receiver, Sender},
  Mutex, RwLock,
};

use crate::{info::write_last_username, AuthStatus, Greeter, Mode};

#[derive(Clone)]
pub struct Ipc(Arc<IpcHandle>);

pub struct IpcHandle {
  tx: RwLock<Sender<Request>>,
  rx: Mutex<Receiver<Request>>,
}

impl Ipc {
  pub fn new() -> Ipc {
    let (tx, rx) = tokio::sync::mpsc::channel::<Request>(10);

    Ipc(Arc::new(IpcHandle {
      tx: RwLock::new(tx),
      rx: Mutex::new(rx),
    }))
  }

  pub async fn send(&self, request: Request) {
    let _ = self.0.tx.read().await.send(request).await;
  }

  pub async fn next(&mut self) -> Option<Request> {
    self.0.rx.lock().await.recv().await
  }

  pub async fn handle(&mut self, greeter: Arc<RwLock<Greeter>>) -> Result<(), Box<dyn Error>> {
    let request = self.next().await;

    if let Some(request) = request {
      let stream = {
        let greeter = greeter.read().await;

        greeter.stream.as_ref().unwrap().clone()
      };

      let response = {
        request.write_to(&mut *stream.write().await).await?;

        Response::read_from(&mut *stream.write().await).await?
      };

      self.parse_response(&mut *greeter.write().await, response).await?;
    }

    Ok(())
  }

  async fn parse_response(&mut self, mut greeter: &mut Greeter, response: Response) -> Result<(), Box<dyn Error>> {
    match response {
      Response::AuthMessage { auth_message_type, auth_message } => match auth_message_type {
        AuthMessageType::Secret => {
          greeter.mode = Mode::Password;
          greeter.working = false;
          greeter.secret = true;
          greeter.set_prompt(&auth_message);
        }

        AuthMessageType::Visible => {
          greeter.mode = Mode::Password;
          greeter.working = false;
          greeter.secret = false;
          greeter.set_prompt(&auth_message);
        }

        AuthMessageType::Error => greeter.message = Some(auth_message),

        AuthMessageType::Info => {
          if let Some(message) = &mut greeter.message {
            message.push_str(&auth_message.trim_end());
            message.push('\n');
          } else {
            greeter.message = Some(auth_message.trim_end().to_string());

            if let Some(message) = &mut greeter.message {
              message.push('\n');
            }
          }

          self.send(Request::PostAuthMessageResponse { response: None }).await;
        }
      },

      Response::Success => {
        if greeter.done {
          if greeter.remember {
            write_last_username(&greeter.username);
          }

          crate::exit(&mut greeter, AuthStatus::Success).await;
        } else if let Some(command) = &greeter.command {
          greeter.done = true;

          self.send(Request::StartSession { cmd: vec![command.clone()] }).await;
        }
      }

      Response::Error { error_type, description } => {
        Ipc::cancel(&mut greeter).await;

        match error_type {
          ErrorType::AuthError => {
            greeter.message = Some(fl!("failed"));
          }

          ErrorType::Error => {
            greeter.message = Some(description);
          }
        }

        greeter.reset().await;
      }
    }

    Ok(())
  }

  pub async fn cancel(greeter: &mut Greeter) {
    let _ = Request::CancelSession.write_to(&mut *greeter.stream().await).await;
  }
}
