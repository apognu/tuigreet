use std::{error::Error, os::unix::net::UnixStream};

use greetd_ipc::{codec::SyncCodec, Request};
use termion::event::Key;

use crate::{
    event::{Event, Events},
    AuthStatus, Greeter, Mode,
};

pub fn handle(
    greeter: &mut Greeter,
    events: &Events,
    stream: &mut UnixStream,
) -> Result<(), Box<dyn Error>> {
    if let Event::Input(input) = events.next()? {
        match input {
            Key::Esc => {
                Request::CancelSession.write_to(stream).unwrap();
                crate::exit(AuthStatus::Success, stream);
            }

            Key::Char('\n') | Key::Char('\t') => {
                greeter.working = true;
                greeter.message = None;

                match greeter.mode {
                    Mode::Username => {
                        greeter.request = Some(Request::CreateSession {
                            username: greeter.username.clone(),
                        });
                    }

                    Mode::Password => {
                        greeter.request = Some(Request::PostAuthMessageResponse {
                            response: Some(greeter.answer.clone()),
                        })
                    }
                }

                greeter.answer = String::new();
            }

            Key::Char(char) => match greeter.mode {
                Mode::Username => greeter.username.push(char),
                Mode::Password => greeter.answer.push(char),
            },

            Key::Backspace => {
                match greeter.mode {
                    Mode::Username => greeter.username.pop(),
                    Mode::Password => greeter.answer.pop(),
                };
            }
            _ => {}
        }
    }

    Ok(())
}
