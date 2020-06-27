use std::{error::Error, os::unix::net::UnixStream};

use greetd_ipc::{codec::SyncCodec, AuthMessageType, ErrorType, Request, Response};

use crate::{AuthStatus, Greeter, Mode};

pub fn handle(greeter: &mut Greeter, stream: &mut UnixStream) -> Result<(), Box<dyn Error>> {
    if let Some(ref mut request) = &mut greeter.request {
        request.write_to(stream)?;
        greeter.request = None;
        let response = Response::read_from(stream)?;

        parse_response(greeter, stream, response)?;
    }

    Ok(())
}

fn parse_response(
    greeter: &mut Greeter,
    stream: &mut UnixStream,
    response: Response,
) -> Result<(), Box<dyn Error>> {
    match response {
        Response::AuthMessage {
            auth_message_type,
            auth_message,
        } => match auth_message_type {
            AuthMessageType::Secret => {
                greeter.mode = Mode::Password;
                greeter.secret = true;
                greeter.prompt = auth_message;
            }

            AuthMessageType::Visible => {
                greeter.mode = Mode::Password;
                greeter.secret = false;
                greeter.prompt = auth_message;
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

                Request::PostAuthMessageResponse { response: None }.write_to(stream)?;
                greeter.request = None;
                let response = Response::read_from(stream)?;

                parse_response(greeter, stream, response)?;
            }
        },

        Response::Success => match greeter.done {
            true => crate::exit(AuthStatus::Success, stream),

            false => {
                greeter.done = true;

                greeter.request = Some(Request::StartSession {
                    cmd: vec![greeter.config().opt_str("cmd").unwrap_or("".to_string())],
                })
            }
        },

        Response::Error {
            error_type,
            description,
        } => match error_type {
            ErrorType::AuthError => {
                crate::exit(AuthStatus::Failure, stream);
            }

            ErrorType::Error => {
                Request::CancelSession.write_to(stream).unwrap();
                greeter.message = Some(description)
            }
        },
    }
    Ok(())
}
