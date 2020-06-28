use std::error::Error;

use greetd_ipc::Request;
use termion::event::Key;

use crate::{
  event::{Event, Events},
  AuthStatus, Greeter, Mode,
};

pub fn handle(greeter: &mut Greeter, events: &Events) -> Result<(), Box<dyn Error>> {
  if let Event::Input(input) = events.next()? {
    match input {
      Key::Esc => crate::exit(greeter, AuthStatus::Cancel)?,

      Key::Left => greeter.cursor_offset -= 1,
      Key::Right => greeter.cursor_offset += 1,

      Key::Ctrl('a') => greeter.cursor_offset = -(greeter.username.len() as i16),
      Key::Ctrl('e') => greeter.cursor_offset = 0,

      Key::Char('\n') | Key::Char('\t') => {
        greeter.working = true;
        greeter.message = None;

        match greeter.mode {
          Mode::Username => {
            if greeter.username.starts_with('!') {
              greeter.command = Some(greeter.username.trim_start_matches("!").to_string());
              greeter.username = String::new();
              greeter.working = false;

              return Ok(());
            }

            greeter.request = Some(Request::CreateSession { username: greeter.username.clone() });
          }

          Mode::Password => {
            greeter.request = Some(Request::PostAuthMessageResponse {
              response: Some(greeter.answer.clone()),
            })
          }
        }

        greeter.answer = String::new();
      }

      Key::Char(c) => insert_key(greeter, c),

      Key::Backspace | Key::Delete => delete_key(greeter, input),

      _ => {}
    }
  }

  Ok(())
}

fn insert_key(greeter: &mut Greeter, c: char) {
  let value = match greeter.mode {
    Mode::Username => &mut greeter.username,
    Mode::Password => &mut greeter.answer,
  };

  let index = value.len() as i16 + greeter.cursor_offset;

  value.insert(index as usize, c);
}

fn delete_key(greeter: &mut Greeter, key: Key) {
  let value = match greeter.mode {
    Mode::Username => &mut greeter.username,
    Mode::Password => &mut greeter.answer,
  };

  let index = match key {
    Key::Backspace => value.len() as i16 + greeter.cursor_offset - 1,
    Key::Delete => value.len() as i16 + greeter.cursor_offset,
    _ => 0,
  };

  if let Some(_) = value.chars().nth(index as usize) {
    value.remove(index as usize);

    if let Key::Delete = key {
      greeter.cursor_offset += 1;
    }
  }
}
