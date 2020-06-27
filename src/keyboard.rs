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

            Key::Left => {
                greeter.cursor_offset -= 1;
            }

            Key::Right => {
                greeter.cursor_offset += 1;
            }

            Key::Char('\n') | Key::Char('\t') => {
                greeter.working = true;
                greeter.message = None;

                match greeter.mode {
                    Mode::Username => {
                        if greeter.username.starts_with('!') {
                            greeter.command =
                                Some(greeter.username.trim_start_matches("!").to_string());
                            greeter.username = String::new();
                            greeter.working = false;
                            return Ok(());
                        }

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
                Mode::Username => {
                    let index = greeter.username.len() as i16 + greeter.cursor_offset;

                    greeter.username.insert(index as usize, char);
                }

                Mode::Password => {
                    let index = greeter.answer.len() as i16 + greeter.cursor_offset;

                    greeter.answer.insert(index as usize, char);
                }
            },

            Key::Backspace => {
                match greeter.mode {
                    Mode::Username => {
                        let index = greeter.username.len() as i16 + greeter.cursor_offset - 1;

                        if let Some(_) = greeter.username.chars().nth(index as usize) {
                            greeter.username.remove(index as usize);
                        }
                    }

                    Mode::Password => {
                        let index = greeter.answer.len() as i16 + greeter.cursor_offset - 1;

                        if let Some(_) = greeter.answer.chars().nth(index as usize) {
                            greeter.answer.remove(index as usize);
                        }
                    }
                };
            }

            Key::Delete => {
                match greeter.mode {
                    Mode::Username => {
                        let index = greeter.username.len() as i16 + greeter.cursor_offset;

                        if let Some(_) = greeter.username.chars().nth(index as usize) {
                            greeter.username.remove(index as usize);
                            greeter.cursor_offset += 1;
                        }
                    }

                    Mode::Password => {
                        let index = greeter.answer.len() as i16 + greeter.cursor_offset;

                        if let Some(_) = greeter.answer.chars().nth(index as usize) {
                            greeter.answer.remove(index as usize);
                            greeter.cursor_offset += 1;
                        }
                    }
                };
            }

            _ => {}
        }
    }

    Ok(())
}
