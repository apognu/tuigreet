use std::error::Error;

use greetd_ipc::{codec::SyncCodec, AuthMessageType, ErrorType, Request, Response};

use crate::{info::write_last_username, AuthStatus, Greeter, Mode};

pub fn handle(greeter: &mut Greeter) -> Result<(), Box<dyn Error>> {
  if let Some(ref request) = greeter.request {
    request.write_to(&mut greeter.stream())?;
    greeter.request = None;
    let response = Response::read_from(&mut greeter.stream())?;

    parse_response(greeter, response)?;
  }

  Ok(())
}

fn parse_response(greeter: &mut Greeter, response: Response) -> Result<(), Box<dyn Error>> {
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

        Request::PostAuthMessageResponse { response: None }.write_to(&mut greeter.stream())?;
        greeter.request = None;
        let response = Response::read_from(&mut greeter.stream())?;

        parse_response(greeter, response)?;
      }
    },

    Response::Success => {
      if greeter.done {
        if greeter.remember {
          write_last_username(&greeter.username);
        }

        crate::exit(greeter, AuthStatus::Success)?;
      } else if let Some(command) = &greeter.command {
        greeter.done = true;
        greeter.request = Some(Request::StartSession { cmd: vec![command.clone()] });
      }
    }

    Response::Error { error_type, description } => {
      cancel(greeter);

      match error_type {
        ErrorType::AuthError => {
          greeter.message = Some(fl!("failed"));
        }

        ErrorType::Error => {
          greeter.message = Some(description);
        }
      }

      greeter.reset();
    }
  }

  Ok(())
}

pub fn cancel(greeter: &mut Greeter) {
  let _ = Request::CancelSession.write_to(&mut greeter.stream());
}
