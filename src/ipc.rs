use std::{error::Error, sync::Arc};

use greetd_ipc::{codec::TokioCodec, AuthMessageType, ErrorType, Request, Response};
use tokio::sync::{
  mpsc::{Receiver, Sender},
  Mutex, RwLock,
};

use crate::{info::write_last_username, AuthStatus, Greeter, Mode};

type IpcChannel = (Arc<Mutex<Receiver<Request>>>, Arc<Mutex<Sender<Request>>>);

pub fn new_ipc() -> IpcChannel {
  let (net_tx, net_rx) = tokio::sync::mpsc::channel::<Request>(10);

  (Arc::new(Mutex::new(net_rx)), Arc::new(Mutex::new(net_tx)))
}

pub async fn handle(greeter: Arc<RwLock<Greeter>>, net_tx: Arc<Mutex<Sender<Request>>>, net_rx: Arc<Mutex<Receiver<Request>>>) -> Result<(), Box<dyn Error>> {
  let request = net_rx.lock().await.recv().await;

  if let Some(request) = request {
    let stream = {
      let greeter = greeter.read().await;

      greeter.stream.as_ref().unwrap().clone()
    };

    let response = {
      request.write_to(&mut *stream.write().await).await?;

      Response::read_from(&mut *stream.write().await).await?
    };

    parse_response(&mut *greeter.write().await, response, net_tx).await?;
  }

  Ok(())
}

async fn parse_response(mut greeter: &mut Greeter, response: Response, net_tx: Arc<Mutex<Sender<Request>>>) -> Result<(), Box<dyn Error>> {
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

        let _ = net_tx.lock().await.send(Request::PostAuthMessageResponse { response: None }).await;
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

        let _ = net_tx.lock().await.send(Request::StartSession { cmd: vec![command.clone()] }).await;
      }
    }

    Response::Error { error_type, description } => {
      cancel(&mut greeter).await;

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
